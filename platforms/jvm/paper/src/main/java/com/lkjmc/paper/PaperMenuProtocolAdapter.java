package com.lkjmc.paper;

import com.lkjmc.common.menu.MenuAction;
import com.lkjmc.common.menu.MenuBundle;
import com.lkjmc.common.menu.MenuController;
import com.lkjmc.common.menu.MenuFrame;
import com.lkjmc.common.menu.MenuRenderer;
import com.lkjmc.common.menu.MenuResult;
import com.lkjmc.common.menu.MenuSnapshotView;
import java.util.Map;

public final class PaperMenuProtocolAdapter {
    private final MenuController controller;

    public PaperMenuProtocolAdapter(MenuBundle bundle, MenuRenderer renderer) {
        controller = new MenuController(bundle, renderer);
    }

    public MenuResult open(long session, String route, Map<String, String> params,
                           String locale, MenuSnapshotView snapshots) {
        return controller.open(session, route, params, locale, snapshots);
    }

    public MenuResult click(MenuFrame.Metadata metadata, MenuAction action, boolean attested) {
        return controller.click(metadata, action, attested);
    }

    public MenuResult response(long request, MenuSnapshotView snapshots) {
        return controller.response(request, snapshots);
    }

    public MenuFrame frame() { return controller.frame(); }
}
