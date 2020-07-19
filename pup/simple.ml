open Pup

let foo = Pup.structure [
    ("type",   U8);
    ("arr_sz", U32); (* must come before the dynamic array *)
    ("arr",    Pup.runtime_array I8 "arr_sz" 10);
    ("buf",    Pup.fixed_array I8 10);
  ]

let frame_timer = Pup.structure [
    ("start", U64);
    ("end",   U64);
  ]

(* Generate a C generic macro for easy logging into a tracer *)

let generate_tracer_helper c_ids_and_types =
  let f (c_id, c_typename) =
    Printf.sprintf "  %s*: tracer_write( (tracer), %s, (message), sizeof( %s ) )"
      c_typename c_id c_typename
  in
  let each_type = List.map f c_ids_and_types in
  Printf.sprintf "#define tracer_write_pup( tracer, message ) _Generic( (message),\\\n%s)\n"
    (String.concat ",\\\n" each_type)

let () =
  Pup.document [("foo", foo); ("frame_timer", frame_timer);]
  |> PupC.create generate_tracer_helper
  |> print_string
