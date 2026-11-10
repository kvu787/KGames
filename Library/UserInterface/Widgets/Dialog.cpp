#include "UserInterface/Widgets/Dialog.h"

#include "Base/Panic.h"

namespace user_interface {

std::string describe_value(DialogResult result) {
    switch (result) {
    case DialogResult::Primary:
        return "Primary";
    case DialogResult::Secondary:
        return "Secondary";
    case DialogResult::Close:
        return "Close";
    }
    base::unreachable();
}

std::string_view dialog_detail::part_name(DialogResult result) {
    switch (result) {
    case DialogResult::Primary:
        return "Primary";
    case DialogResult::Secondary:
        return "Secondary";
    case DialogResult::Close:
        return "Close";
    }
    base::unreachable();
}

float dialog_detail::gap(float height) {
    return height > 0.0f ? controls::DialogTitleGap : 0.0f;
}

} // namespace user_interface
