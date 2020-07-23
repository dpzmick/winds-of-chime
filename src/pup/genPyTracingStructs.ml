open PupCore

let extra =
  Printf.sprintf "PUP_STRUCT_IDS = {\n%s\n}"
    (String.concat ",\n"
       (PupPy.py_map_ids
          (fun id name -> Printf.sprintf "  %d: %s" id name)
          TracingStructs.doc))

let () =
  PupPy.create_with_extra TracingStructs.doc extra
  |> print_string
