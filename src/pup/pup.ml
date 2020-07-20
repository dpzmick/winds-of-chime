open PupTypes

(* concrete types *)
type pup_structure = (string * pup_type) list
type pup_document = (string * pup_structure) list

(* helpers *)
let fixed_array pt sz =
  Array (pt, FixedSize (Constant sz))

let runtime_array pt field max =
  Array (pt, RuntimeSize ((Member field), (Constant max)))

(* FIXME perform validation *)
let structure members = members

let map_values s f = List.map (fun (n, t) -> f n t) s

let rec comp_time_eval expr =
  match expr with
  | Constant c -> c
  | Add (a, b) ->
     (comp_time_eval a) + (comp_time_eval b)
  | Mul (a, b) ->
     (comp_time_eval a) * (comp_time_eval b)
  | Member _  ->
     raise (Invalid_argument "expr cannot be evaulated at compile time")

let rec type_size pt =
  match pt with
  | I8 | U8   -> Constant 1
  | I32 | U32 -> Constant 4
  | I64 | U64 -> Constant 8
  | Array (pt, (FixedSize expr)) ->
     Mul (type_size pt, expr)
  | Array (pt, (RuntimeSize (expr, _))) ->
     Mul (type_size pt, expr)

let type_max_size pt =
  (match pt with
   | Array (pt, (RuntimeSize (_, expr))) ->
      Mul (type_size pt, expr)
   | _ -> type_size pt)
  |> comp_time_eval

let structure_offsets s =
  let rec f (acc : (string * expr) list) last_expr rest =
    match rest with
    | [] -> acc
    | (name, pt)::rest ->
       let expr = Add (last_expr, type_size pt) in
       f ((name, last_expr)::acc) expr rest
  in
  f [] (Constant 0) s

let structure_max_size s =
  let f acc e = acc + (type_max_size e) in
  List.fold_left f 0 (List.map snd s)

let document lst = lst

let map_structs doc f =
  List.map (fun (n, m) -> f n m) doc

let fold_structs doc init f =
  List.fold_left (fun acc (n, m) -> f acc n m) init doc

let structure_ids doc =
  fold_structs doc (0, [])
    (fun (id, acc) name _ ->
       (id + 1, acc@[(name, id)]))
  |> snd

(* make sure all dynamically sized fields are referenced before other fields *)
(* let validate_document doc = false *)
(* FIXME allow types to refer to other types? Doesn't seem actually that useful for something like tracing *)
