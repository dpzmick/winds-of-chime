open PupCore.PupTypes
open PupCore.Pup

let ticktock = structure [("start",  U64);
                          ("end",    U64);
                          ("tag_sz", U64);
                          ("tag",    runtime_array I8 "tag_sz" 64);]  (* null-terminated string *)

let next_image = structure [("next_image_idx", U32);]

let test = structure [("a_sz", U64);
                      ("cnt", U64);
                      ("arr", Array
                         (Array (U64,
                                 (RuntimeSize (Member "a_sz", Constant 10))),
                          (RuntimeSize (Member "cnt", Constant 10))));]

let doc = document [("ticktock", ticktock);
                    ("next_image", next_image);]
                    (* ("test", test)] *)
