open PupTypes
open Pup

(* python is a bit of a pain b.c. I hav to write a load and store api *)

let py_key ty =
  match ty with
  | I8 -> "b"
  | U8 -> "B"
  | I32 -> "i"
  | U32 -> "I"
  | I64 -> "q"
  | U64 -> "Q"
  | Array _ -> raise (Invalid_argument "cannot pack/unpack array types")

let rec py_expr expr =
  match expr with
  | Member name -> name
  | Constant i  -> Printf.sprintf "%d" i
  | Add (a, b)  -> Printf.sprintf "(%s) + (%s)" (py_expr a) (py_expr b)
  | Mul (a, b)  -> Printf.sprintf "(%s) * (%s)" (py_expr a) (py_expr b)

let snake_to_camel str =
  let str = String.lowercase_ascii str in
  let parts = String.split_on_char '_' str in
  let parts = List.map String.capitalize_ascii parts in
  String.concat "" parts

let py_make_init s =
  let args = String.concat ", " (map_values s (fun name _ -> name)) in
  let assignments = String.concat "\n"
                      (map_values s (fun name _ -> Printf.sprintf "    self.%s = %s" name name))
  in
  Printf.sprintf "  def __init__(self, %s):\n%s\n" args assignments

let py_load offsets name typ =
  let offset_expr = List.assoc name offsets in
  match typ with
  | Array (pt, (FixedSize size_expr)) ->
     Printf.sprintf "    %s = []\n    for i in range(0, %s):\n      tmp = struct.struct('=%s', b[(%s):(%s)])\n      %s.append(tmp)"
       name
       (py_key pt)
       (py_expr offset_expr)
       (py_expr (Add (offset_expr, (Mul (size_expr, Member "i")))))
       (py_expr size_expr)
       name
  | Array (pt, (RuntimeSize (array_size_expr, _))) ->
     let size_expr = type_size pt in
     let array_offset_expr = (Add (offset_expr, (Mul (size_expr, Member "i")))) in
     Printf.sprintf "    %s = []\n    for i in range(0, %s):\n      tmp = struct.unpack('=%s', b[(%s):(%s)])\n      %s.append(tmp)"
       name
       (py_expr array_size_expr)
       (py_key pt)
       (py_expr array_offset_expr)
       (py_expr (Add (array_offset_expr, size_expr)))
       name
  | _ ->
     let size_expr = type_size typ in
     Printf.sprintf "    (%s,) = struct.unpack('=%s', b[(%s):(%s)])"
           name
           (py_key typ)
           (py_expr offset_expr)
           (py_expr (Add (offset_expr, size_expr)))

let py_make_pack _s =
  Printf.sprintf "  def pack(self):\n    pass\n"

let py_make_unpack s offsets =
  Printf.sprintf "  @classmethod\n  def unpack(cls, b):\n%s\n    return cls(%s)\n"
    (String.concat "\n" (map_values s (fun name typ -> py_load offsets name typ)))
    (String.concat ", " (map_values s (fun name _ -> name)))

let py_str struct_name s =
  Printf.sprintf "  def __str__(self):\n   return f'%s(%s)'"
    struct_name
    (String.concat ","
       (map_values s (fun name _ -> Printf.sprintf "%s={self.%s}" name name)))

let py_make_class struct_name s =
  let offsets = structure_offsets s in
  Printf.sprintf "class %s:\n%s\n%s\n\n%s\n%s\n%s\n"
    (snake_to_camel struct_name)
    (Printf.sprintf "  __slots__ = [%s]" (String.concat ", " (map_values s (fun name _ -> "'" ^ name ^ "'"))))
    (py_make_unpack s offsets)
    (py_make_init s)
    (py_make_pack s)
    (py_str (snake_to_camel struct_name) s)

let py_map_ids f doc =
  List.map
    (fun (struct_name, id) -> f id (snake_to_camel struct_name))
    (structure_ids doc)

let create_with_extra doc extra =
  Printf.sprintf "import struct\n%s\n%s"
    (String.concat "\n"
       (map_structs doc py_make_class))
    extra

let create doc = create_with_extra doc ""

(* generated have explit pack/unpack methods to avoid repeated boxing/unboxing
   into python.

   This isn't ideal, but there isn't really an ideal scenario for python.
   Trying to convert straight to numpy isn't possible b.c. records would be
   variable lengths.

   It might be possible to write extractors that walk the input file
   extracting some subset of the fields (where some condition holds) and
   convert those to numpy arrays *)

(* FIXME this doesn't support nested arrays, but could with a bit of work *)

(* FIXME implement pack *)
