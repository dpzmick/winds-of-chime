open PupTypes
open Pup

let rec c_signature_const pt =
  match pt with
  | I8  -> "const int8_t"
  | U8  -> "const uint8_t"
  | I32 -> "const int32_t"
  | U32 -> "const uint32_t"
  | I64 -> "const int64_t"
  | U64 -> "const uint64_t"
  | Array (pt, _) -> Printf.sprintf "%s * const" (c_signature_const pt)

let rec c_signature pt =
  match pt with
  | I8  -> "int8_t"
  | U8  -> "uint8_t"
  | I32 -> "int32_t"
  | U32 -> "uint32_t"
  | I64 -> "int64_t"
  | U64 -> "uint64_t"
  | Array (pt, _) -> Printf.sprintf "%s *" (c_signature pt)

(* there's no such thing as a simple expression in C unfortunately.
   If there's an array size of uint32_t * uint32_t, users need to be careful to
   not overflow the array size calculation expression.

   For this reason we convert all internal values used to compute sizes and
   offsets into 64 bit values. If a value computed by an expr needs to be
   returned to the user, it will be converted to user sized type at the last
   second. *)

let rec c_simple_expr expr =
  match expr with
  | Member field_name -> Printf.sprintf "(uint64_t)%s" field_name
  | Constant v -> Printf.sprintf "%dul" v
  | Add (a, b) -> Printf.sprintf "(%s) + (%s)" (c_simple_expr a) (c_simple_expr b)
  | Mul (a, b) -> Printf.sprintf "(%s) * (%s)" (c_simple_expr a) (c_simple_expr b)

let c_make_struct_typename = Printf.sprintf "%s_t"
let c_make_struct_varname x = x (* identity for now *)

let c_make_struct_ptr_arg struct_name =
  (Printf.sprintf "%s_t *" struct_name, struct_name)

let c_make_const_struct_ptr_arg struct_name =
  (Printf.sprintf "%s_t const *" struct_name, struct_name)

let c_make_enum_name struct_name =
  Printf.sprintf "PUP_%s_ID" (String.uppercase_ascii struct_name)

(* args is pairs of c_type, c_name *)
let c_make_signature c_return_type fname args =
  let print (c_type, c_name) = Printf.sprintf "%s %s" c_type c_name in
  Printf.sprintf "static inline %s %s( %s )"
    c_return_type
    fname
    (String.concat ", " (List.map print args))

(* dst is always pointer, src is always "value" *)
let c_make_copy typ src_value dst_ptr =
  match typ with
  | Array _ ->
    Printf.sprintf "memcpy( %s, %s, %s );"
      dst_ptr
      src_value
      (c_simple_expr (type_size typ))
  (* only difference is the & in src *)
  | _ ->
    Printf.sprintf "memcpy( %s, &%s, (%s) );"
      dst_ptr
      src_value
      (c_simple_expr (type_size typ))

let c_struct_defn s struct_name =
  Printf.sprintf
    "typedef struct {\n  char buffer[%d];\n} %s;\n"
    (structure_max_size s)
    (c_make_struct_typename struct_name)

let c_get_buffer = Printf.sprintf "%s->buffer"

let c_make_reset s offsets struct_name =
  (* one argument per member *)
  let argf name typ = (c_signature_const typ, name) in
  let arguments = (c_make_struct_ptr_arg struct_name)::(map_values s argf) in

  (* one store for each field *)
  let target name =
    Printf.sprintf "%s + %s"
      (c_get_buffer (c_make_struct_varname struct_name))
      (c_simple_expr (List.assoc name offsets))
  in
  let storef name typ = c_make_copy typ name (target name) in
  let stores : string list = map_values s storef in

  Printf.sprintf
    "%s {\n%s\n}\n"
    (c_make_signature
       "void"
       (Printf.sprintf "%s_reset" struct_name)
       arguments)
    (String.concat "\n"
       (List.map (Printf.sprintf "  %s") stores))

(* figure out the list of members needed to compute an offset *)
let rec dependent_members offset_expr =
  match offset_expr with
  | Constant _ -> []
  | Member name -> [name]
  | Add (a, b) -> (dependent_members a)@(dependent_members b)
  | Mul (a, b) -> (dependent_members a)@(dependent_members b)

let c_getter_sig struct_name member_name member_type =
  let fn = Printf.sprintf "%s_get_%s" struct_name member_name in
  match member_type with
  (* always return array size as 64 bit number. User can cast if they know more than us *)
  | Array _ ->
    c_make_signature
      "void"
      fn
      [(c_make_const_struct_ptr_arg struct_name);
       ((c_signature member_type), "out_array");
       ("uint64_t *", "out_array_size")]
  | _ ->
    c_make_signature
      (c_signature member_type)
      fn
      [(c_make_const_struct_ptr_arg struct_name);]

let c_getter offsets struct_name member_name member_type =
  let signature = c_getter_sig struct_name member_name member_type in
  let varname = (c_make_struct_varname struct_name) in
  let offset_expr = List.assoc member_name offsets in
  let src_c_expr = Printf.sprintf
      "%s + %s"
      (c_get_buffer varname)
      (c_simple_expr offset_expr)
  in
  (* load all dependent fields. all deps must be other fields. We can cast them
     all to uint64_t because are computing an offset (and to avoid type
     lookups) *)
  let deps = (dependent_members offset_expr)@(dependent_members (type_size member_type)) in
  let load_expr member_name =
      Printf.sprintf "uint64_t %s = (uint64_t)%s_get_%s( %s );"
        member_name
        struct_name
        member_name
        varname
  in
  let loaders = List.map load_expr deps in
  (* create return value *)
  let exprs = match member_type with
    | Array (_, (FixedSize sz)) ->
      (* load into outparams, return nothing *)
      [(c_make_copy member_type src_c_expr "out_array");
       (Printf.sprintf "*out_array_size = %s;" (c_simple_expr sz));
      ]
    | Array (_, (RuntimeSize (sz, _))) ->
      (* load into outparams, return nothing *)
      [(c_make_copy member_type src_c_expr "out_array");
       (Printf.sprintf "*out_array_size = %s;" (c_simple_expr sz));
      ]
    | _ ->
      (* load into temp value, return temp value *)
      [(Printf.sprintf "%s ret;" (c_signature member_type));
       (c_make_copy member_type src_c_expr "&ret");
       "return ret;";
      ]
  in
  (* put it all together *)
  let body = List.map (Printf.sprintf "  %s") (loaders @ exprs) in
  Printf.sprintf "%s {\n%s\n}\n"
    signature
    (String.concat "\n" body)

let c_make_size_fn s struct_name =
  let varname = (c_make_struct_varname struct_name) in
  let arguments = [(c_make_const_struct_ptr_arg struct_name)] in
  let exprs = map_values s (fun _ ty -> type_size ty) in
  let expr = List.fold_left
      (fun acc el -> Add (acc, el))
      (Constant 0)
      exprs
  in
  let deps = dependent_members expr in
  let load_expr member_name =
      Printf.sprintf "uint64_t %s = (uint64_t)%s_get_%s( %s );"
        member_name
        struct_name
        member_name
        varname
  in
  let loaders = List.map load_expr deps in
  let body = loaders@[
      "return " ^ (c_simple_expr expr) ^ ";";
    ]
  in
  Printf.sprintf
    "%s {\n%s\n}\n"
    (c_make_signature
       "uint64_t"
       (Printf.sprintf "%s_size" struct_name)
       arguments)
    (String.concat "\n" (List.map (Printf.sprintf "  %s") body))

let c_make_methods s struct_name =
  let offsets = structure_offsets s in
  let reset = c_make_reset s offsets struct_name in
  let getters = map_values s (fun name typ -> c_getter offsets struct_name name typ) in
  let sz = c_make_size_fn s struct_name in
  String.concat "\n" ([reset]@getters@[sz])

(* for users to generate their own additions, they will need to know all of the
   names we've generated. For now, we provide IDs and types *)

let c_map_ids f doc =
  List.map (fun (name, _) -> f (c_make_enum_name name) (c_make_struct_typename name)) (structure_ids doc)

let base_includes = [
  "stdint.h";
  "stdbool.h";
  "string.h";
]

let create_with_extra (doc : pup_document) extra =
  let flat s = (String.concat "\n" s) ^ "\n" in
  flat [
    (* includes *)
    flat
      (List.map (Printf.sprintf "#include <%s>") base_includes);

    (* c++ *)
    "#ifdef __cplusplus\nextern \"C\" {\n#endif\n";

    (* IDs *)
    "enum {";
    List.map
      (fun (name, id) -> Printf.sprintf "  %s = %d," (c_make_enum_name name) id)
      (structure_ids doc)
    |> String.concat "\n";
    "};\n";

    (* type definitions *)
    flat
      (map_structs doc (fun name s -> c_struct_defn s name));

    (* function definitions *)
    flat
      (map_structs doc (fun name s -> c_make_methods s name));

    extra;

    (* /c++ *)
    "#ifdef __cplusplus\n}\n#endif";
  ]

let create (doc : pup_document) = create_with_extra doc ""

(* No setters allowed! Changing the size of a runtime sized array would change
   the layout and require moving all downstream fields. Feels like a pain. *)

(* loader api is just casting something to a struct and reading fields *)
