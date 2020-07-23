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

let rec print_expr expr =
  match expr with
  | Member m -> m
  | Constant c -> string_of_int c
  | Add (a, b) -> "(" ^ (print_expr a) ^ ") + (" ^ (print_expr b) ^ ")"
  | Mul (a, b) -> "(" ^ (print_expr a) ^ ") * (" ^ (print_expr b) ^ ")"

(* let rec print_type pt =
 *   match pt with
 *   | I8 -> "I8"
 *   | U8 -> "U8"
 *   | I32 -> "I8"
 *   | U32 -> "I32"
 *   | I64 -> "I8"
 *   | U64 -> "I64"
 *   | Array (pt, (RuntimeSize (a, b))) -> Printf.sprintf "Array (%s) (RuntimeSize %s,%s)"
 *                                           (print_type pt)
 *                                           (print_expr a)
 *                                           (print_expr b)
 *   | Array (pt, (FixedSize a)) -> Printf.sprintf "Array (%s) (FixedSize %s)"
 *                                    (print_type pt)
 *                                    (print_expr a) *)

let rec comp_time_eval expr =
  match expr with
  | Constant c -> c
  | Add (a, b) ->
     (comp_time_eval a) + (comp_time_eval b)
  | Mul (a, b) ->
     (comp_time_eval a) * (comp_time_eval b)
  | Member _  ->
     raise (Invalid_argument ("expr " ^ (print_expr expr) ^ " cannot be evaluated at compile time"))

let rec type_size pt =
  match pt with
  | I8 | U8   -> Constant 1
  | I32 | U32 -> Constant 4
  | I64 | U64 -> Constant 8
  | Array (pt, (FixedSize expr)) ->
     Mul (type_size pt, expr)
  | Array (pt, (RuntimeSize (expr, _))) ->
     Mul (type_size pt, expr)

let rec type_max_size pt =
  match pt with
  | Array (pt, (FixedSize expr)) ->
    (type_max_size pt) * (comp_time_eval expr)

  | Array (pt, (RuntimeSize (_, expr))) ->
    (type_max_size pt * (comp_time_eval expr))

  | _ -> comp_time_eval (type_size pt)

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
