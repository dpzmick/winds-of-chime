(* expression used for offsets and sizes of fields *)
type expr =
  | Member of string
  | Constant of int
  | Add of expr * expr
  | Mul of expr * expr

(* qualifier for the types of arrays that can be created *)
type array_qualifier =
  | FixedSize of expr
  | RuntimeSize of expr * expr  (* expression for runtime size, expression for max size *)

(* types supported for a field *)
type pup_type =
  | I8
  | U8
  | I32
  | U32
  | I64
  | U64
  | Array of pup_type * array_qualifier

(* a structure is a list of named fields (each field has a single type) *)
type pup_structure

(* a document is a list of named strutures *)
type pup_document

(* create a structure from an associative array *)
val structure : (string * pup_type) list -> pup_structure

val member_names : pup_structure -> string list
val member_type : pup_structure -> string -> pup_type
val member_offset : pup_structure -> string -> expr
val member_size : pup_structure -> string -> expr
val member_max_size : pup_structure -> string -> int

(* create a document from an associative array *)
val document : (string * pup_structure) list -> pup_document

val struct_names : pup_document -> string list
val struct_type : pup_document -> string -> pup_structure

(* helpers for easy construction *)
val fixed_array : pup_type -> int -> pup_type
val runtime_array : pup_type -> string -> int
