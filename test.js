/* This file mainly serves the purpose of testing if we can correctly import the lib through GJS.
DO NOT MOVE IT as it is referenced in the test.sh Bash file */

import GtkGlShaders from "gi://GtkGlShaders";
print(GtkGlShaders.test_obj_new());
