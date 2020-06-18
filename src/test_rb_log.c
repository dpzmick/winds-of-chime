#include "rb_log.h"
#include "common.h"

#include <stdio.h>

int main() {
  rb_schema_t * schema = new_rb_schema();
  rb_schema_add_object( "obj1" );
  rb_schema_add_object( "obj2" );
}
