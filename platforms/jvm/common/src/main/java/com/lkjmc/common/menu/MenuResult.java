package com.lkjmc.common.menu;

public sealed interface MenuResult permits MenuResult.Rendered, MenuResult.Closed,
        MenuResult.Failed, MenuResult.Ignored {
    record Rendered(MenuFrame frame) implements MenuResult {}
    record Closed() implements MenuResult {}
    record Failed(MenuTypes.Failure failure, String message) implements MenuResult {}
    record Ignored() implements MenuResult {}
}
