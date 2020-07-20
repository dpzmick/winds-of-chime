open PupTypes

(* a structure is a list of named fields (each field has a single type) *)
type pup_structure

(* a document is a list of named strutures *)
type pup_document

(* create expression for size type in bytes *)
val type_size : pup_type -> expr

(* create a structure from an associative array *)
val structure : (string * pup_type) list -> pup_structure

(* iterate over all members *)
val map_values : pup_structure -> (string -> pup_type -> 'a) -> 'a list

(* compute values over entire structure *)
val structure_max_size : pup_structure -> int
val structure_offsets : pup_structure -> (string * expr) list

(* create a document from an associative array *)
val document : (string * pup_structure) list -> pup_document
val map_structs : pup_document -> (string -> pup_structure -> 'a) -> 'a list
val fold_structs : pup_document -> 'a -> ('a -> string -> pup_structure -> 'a) -> 'a

(* assign unique ids to each structure in the document *)
val structure_ids : pup_document -> (string * int) list

(* helpers for easy construction *)
val fixed_array : pup_type -> int -> pup_type
val runtime_array : pup_type -> string -> int -> pup_type
