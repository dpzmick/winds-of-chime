open PupCore.PupTypes
open PupCore.Pup

let ticktock = structure [("start",  U64);
                          ("end",    U64);
                          ("tag_sz", U64);
                          ("tag",    runtime_array I8 "tag_sz" 64);]  (* null-terminated string *)

let next_image = structure [("next_image_idx", U32);]

let doc = document [("ticktock", ticktock);
                    ("next_image", next_image);]
