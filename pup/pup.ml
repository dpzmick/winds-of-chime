(* concrete types *)
type pup_struct = (string * ptype) list
type pup_document = (string * pstruct) list

(* helpers *)
let fixed_array pt sz =
  Array (pt, FixedSize (Constant sz))

let runtime_array pt field max =
  Array (pt, RuntimeSize ((Member field), (Constant max)))

(* FIXME perform validation *)
let structure members = members

let member_names s = List.map fst s
let member_types s = List.assoc s

let rec member_size pt =
  match pt with
  | I8 | U8   ->
     Constant 1
  | I32 | U32 ->
     Constant 4
  | I64 | U64 ->
     Constant 8
  | Array (pt, (FixedSize expr)) ->
     Mul (member_size pt, expr)
  | Array (pt, (RuntimeSize (expr, _))) ->
     Mul (member_size pt, expr)

let rec comp_time_eval expr =
  match expr with
  | Constant c -> c
  | Add (a, b) ->
     (comp_time_eval a) + (comp_time_eval b)
  | Mul (a, b) ->
     (comp_time_eval a) * (comp_time_eval b)
  | Member _  ->
     raise (Invalid_argument "expr cannot be evaulated at compile time")

let max_size pt =
  (match pt with
   | Array (pt, (RuntimeSize (_, expr))) ->
      Mul (member_size pt, expr)
   | _ -> member_size pt)
  |> comp_time_eval

let compute_offsets members =
  let func (last, acc) (name, pt) =
    let this = match last with
      | None -> Constant 0
      | Some (last_name, last_pt) ->
         Add ((List.assoc last_name acc), (member_size last_pt))
    in
    (Some (name, pt), (name, this)::acc)
  in
  List.fold_left func (None, []) members
  |> snd
  |> List.rev


(* make sure all dynamically sized fields are referenced before other fields *)
(* let validate_document doc = false *)

(* FIXME allow types to refer to other types? Doesn't seem actually that useful for something like tracing *)

(* FIXME perform validation *)
let document structs : pdocument = structs
