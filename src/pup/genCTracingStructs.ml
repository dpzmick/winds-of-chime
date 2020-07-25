open PupCore

(* Generate a C generic macro for easy logging into a tracer *)

let c_generate_tracer_helper doc =
  let variant c_id_val c_type_name =
    Printf.sprintf "  %s*: tracer_write( (tracer), %s, (message), %s_size( (void const*)(message) ) )"
      c_type_name
      c_id_val
      (String.sub c_type_name 0 ((String.length c_type_name)-2))
  in
  Printf.sprintf "#define tracer_write_pup( tracer, message ) _Generic( (message),\\\n%s )\n"
    (String.concat ", \\\n" (PupC.c_map_ids variant doc))

let () =
  PupC.create_with_extra TracingStructs.doc (c_generate_tracer_helper TracingStructs.doc)
  |> print_string


(* FIXME use the size function instead of sizeof() for write to tracer *)
