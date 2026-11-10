// The mesh modeler. See README.md.

#include "MeshModeler/MeshModeler.h"
#include "UserInterface/UserInterface.h"

#include "Folders.h"

int main() {
    return user_interface::run(user_interface::Options("MeshModeler", folders::Project),
                               [](user_interface::Startup& startup) { return mesh_modeler::modeler::Modeler::start(startup); });
}
