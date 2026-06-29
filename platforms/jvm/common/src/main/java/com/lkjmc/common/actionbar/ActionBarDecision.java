package com.lkjmc.common.actionbar;

import java.util.Optional;

public record ActionBarDecision(ActionBarState state, Optional<ActionBarFrame> frame) {}
