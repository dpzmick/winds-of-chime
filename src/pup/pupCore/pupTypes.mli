(* common types used across the pup system *)

(* expression used for offsets and sizes of fields *)
type expr =
  | Member of string
  | Constant of int
  | Add of expr * expr
  | Mul of expr * expr

(* qualifier for the types of arrays that can be created *)
type array_qualifier =
  | FixedSize of expr
  | RuntimeSize of expr * expr  (* size, max_size *)

(* types supported for a field *)
type pup_type =
  | I8
  | U8
  (* FIXME 16 bit *)
  | I32
  | U32
  | I64
  | U64
  | Array of pup_type * array_qualifier
