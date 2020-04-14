extern crate vkxml;

fn print<T: std::fmt::Debug>(t: T) {
    println!("got {:#?}", t);
}

fn main() {
    let filename = "vk.xml";
    let contents = vkxml::get_file_contents(filename).expect("file");
    let doc = vkxml::Document::for_file_contents(&contents);

    vkxml::Parser::for_document(&doc)
        .on_platform(print)
        .on_tag(print)
        .on_basetype(print)
        .on_bitmask_definition(print)
        .on_bitmask_alias(print)
        .on_handle(print)
        .on_handle_alias(print)
        .on_enum_definition(print)
        .on_enum_alias(print)
        .on_function_pointer(print)
        .on_struct(print)
        .on_union(print)
        .on_enum(print)
        .on_command(print)
        .on_command_alias(print)
        .on_feature(print)
        .on_extension(print)
        .parse_document();
}