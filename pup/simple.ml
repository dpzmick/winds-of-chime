open PupTypes

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

let c_generate_tracer_helper doc =
  let variant c_id_val c_type_name =
    Printf.sprintf "  %s*: tracer_write( (tracer), %s, (message), sizeof( %s ) )"
      c_type_name
      c_id_val
      c_type_name
  in
  Printf.sprintf "#define tracer_write_pup( tracer, message ) _Generic( (message),\\\n%s )\n"
    (String.concat ", \\\n" (PupC.c_map_ids variant doc))

let doc = Pup.document [("foo", foo);
                        ("frame_timer", frame_timer);]

let () =
  PupC.create_with_extra doc (c_generate_tracer_helper doc)
  |> print_string
