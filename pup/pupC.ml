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

let rec c_simple_expr expr =
  match expr with
  | Member field_name -> field_name
  | Constant v -> Printf.sprintf "%d" v
  | Add (a, b) -> Printf.sprintf "%s + %s" (c_simple_expr a) (c_simple_expr b)
  | Mul (a, b) -> Printf.sprintf "%s * %s" (c_simple_expr a) (c_simple_expr b)

let c_get_offset member_name members =
  (c_simple_expr (List.assoc member_name (compute_offsets members)))

let c_struct_defn name members =
  let types = List.map snd members in
  let f acc el = acc + (max_size el) in
  let max_sz = List.fold_left f 0 types in
  Printf.sprintf
    "typedef struct {\n  char buffer[%d];\n} %s_t;"
    max_sz
    name

let c_struct_make_init struct_name members =
  let mk_member_signature (name, pt) =
    Printf.sprintf "%s %s" (c_signature_const pt) name
  in
  let whole_signature =
    Printf.sprintf "static inline void %s_init( %s_t * %s, %s )"
      struct_name struct_name struct_name
      (String.concat ", " (List.map mk_member_signature members))
  in
  let copies_section =
    let printer (name, pt) =
      let off = (c_get_offset name members) in
      match pt with
      | Array (_, (FixedSize expr)) ->
        Printf.sprintf "  memcpy( %s->buffer + %s, %s, sizeof( *%s ) * %s )"
          struct_name
          off
          name
          name
          (c_simple_expr expr)
      | Array (_, (RuntimeSize (expr, _))) ->
        Printf.sprintf "  memcpy( %s->buffer + %s, %s, sizeof( *%s ) * %s )"
          struct_name
          off
          name
          name
          (c_simple_expr expr)
      | _ ->
        Printf.sprintf "  memcpy( %s->buffer + %s, &%s, sizeof( %s ) )"
          struct_name
          off
          name
          name
    in String.concat ";\n" (List.map printer members) ^ ";\n"
  in
  Printf.sprintf
    "%s {\n%s\n}"
    whole_signature
    copies_section

(* FIXME move dependent field logic into core *)

(* load *all* of the fields needed for runtime offset calculation.
 * depend on dead code elimination to remove the unneeded loads FIXME check if
 * it works *)

(* NOTE: only supporting runtime-sized arrays which use exactly a single member *)
let c_load_dependent_fields struct_name members except =
  let f member = match member with
    | (_, Array (_, RuntimeSize ((Member name), _))) -> name <> except
    | _ -> false
  in
  let dependent = List.filter f members in
  let f member = match member with
    | (_, Array (_, RuntimeSize ((Member ref_member_name), _))) ->
      Printf.sprintf "  %s %s = %s_get_%s( %s ); (void)%s;"
        (c_signature_const (List.assoc ref_member_name members))
        ref_member_name
        struct_name
        ref_member_name
        struct_name
        ref_member_name (* silence unused variable warning *)
    | _ -> raise (Failure "unreachable")
  in
  let loaders = List.map f dependent in
  String.concat ";\n" loaders

let c_getter_sig struct_name members (member_name, member_type) =
  match member_type with
  | Array (_, (FixedSize _)) ->
    Printf.sprintf "static inline void %s_get_%s( %s_t * %s, %s out_%s )"
      struct_name
      member_name
      struct_name
      struct_name
      (c_signature member_type)
      member_name
  | Array (_, (RuntimeSize ((Member ref_member_name), _))) ->
    Printf.sprintf "static inline void %s_get_%s( %s_t * %s, %s out_%s, %s * out_%s )"
      struct_name
      member_name
      struct_name
      struct_name
      (c_signature member_type)
      member_name
      (c_signature (List.assoc ref_member_name members))
      ref_member_name
  | _ ->
    Printf.sprintf "static inline %s %s_get_%s( %s_t const * %s )"
      (c_signature member_type)
      struct_name
      member_name
      struct_name
      struct_name

let c_getter struct_name members (member_name, member_type) =
  let signature = c_getter_sig struct_name members (member_name, member_type) in
  match member_type with
  | Array (_, (FixedSize expr)) ->
    Printf.sprintf "%s {\n%s\n%s\n}"
      signature
      (c_load_dependent_fields struct_name members member_name)
      (Printf.sprintf "  memcpy( out_%s, %s->buffer + %s, sizeof( *out_%s ) * %s );"
         member_name
         struct_name
         (c_get_offset member_name members)
         member_name
         (c_simple_expr expr))
  | Array (_, (RuntimeSize (expr, _))) ->
    Printf.sprintf "%s {\n%s\n%s\n%s\n}"
      signature
      (c_load_dependent_fields struct_name members member_name)
      (Printf.sprintf "  memcpy( out_%s, %s->buffer + %s, sizeof( *out_%s ) * %s );"
         member_name
         struct_name
         (c_get_offset member_name members)
         member_name
         (c_simple_expr expr))
      (Printf.sprintf "  *out_%s = %s;"
         (* bit of a hack, but we know these are just of type (Member) *)
         (c_simple_expr expr)
         (c_simple_expr expr))
  | _ ->
    Printf.sprintf "%s {\n%s\n%s\n}"
      signature
      (c_load_dependent_fields struct_name members member_name)
      (Printf.sprintf "  %s ret;\n  memcpy( &ret, %s->buffer + %s, sizeof( ret ) );\n  return ret;"
         (c_signature member_type)
         struct_name
         (c_get_offset member_name members))

(* No setters allowed because changing the size of a runtime sized array would
   change the layout and require moving all downstream fields. Users can write
   their own wrappers to do this explicitly. *)

let c_struct_decls name members =
  let getters = String.concat ";\n" (List.map (c_getter_sig name members) members) in
  Printf.sprintf "%s;\n"
    getters

let c_struct_methods name members =
  let getters = String.concat "\n" (List.map (c_getter name members) members) in
  Printf.sprintf "%s\n%s\n"
    (c_struct_make_init name members)
    getters

(* Generate ids for tagging messages if needed *)
let c_make_ids doc =
  let f i (nm, _) = Printf.sprintf "  PUP_%s_ID = %d," (String.uppercase_ascii nm) i in
  let ids = String.concat "\n" (List.mapi f doc) in
  Printf.sprintf "enum {\n%s\n};\n" ids

let create extra (doc : pdocument) =
  let includes = ["stdint.h"; "stdbool.h"; "string.h"] in
  let includes = String.concat "\n" (List.map (Printf.sprintf "#include <%s>") includes) in
  let type_definitions = String.concat "\n\n"
      (List.map (fun (name, members) -> c_struct_defn name members) doc)
  in
  let declarations = String.concat "\n\n"
      (List.map (fun (name, members) -> c_struct_decls name members) doc)
  in
  let methods = String.concat "\n\n"
      (List.map (fun (name, members) -> c_struct_methods name members) doc)
  in
  let extras =
    let assoc_c_ids_names = [("PUP_FOO_ID", "foo_t"); ("PUP_FRAME_TIMER_ID", "frame_timer_t");] in
    extra assoc_c_ids_names
  in
  String.concat "\n\n"
    [includes;
     "#ifdef __cplusplus\nextern \"C\" {\n#endif";
     (c_make_ids doc);
     type_definitions;
     declarations;
     methods;
     extras;
     "#ifdef __cplusplus\n}\n#endif";]

(* bad field references will cause compiler error in c compiler for now *)
