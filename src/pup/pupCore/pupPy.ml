open PupTypes
open Pup

(* python is a bit of a pain b.c. I hav to write a load and store api *)

let py_struct_key ty =
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

let py_make_class struct_name _s =
  Printf.sprintf "class %s:\n  pass"
    (snake_to_camel struct_name)

let create doc =
  String.concat "\n"
    (map_structs doc py_make_class)

(* generated have explit pack/unpack methods to avoid repeated boxing/unboxing
   into python. *)
