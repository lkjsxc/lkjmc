package com.lkjmc.bindings;

import com.google.gson.annotations.SerializedName;

public enum GameMode {
    @SerializedName("survival") SURVIVAL,
    @SerializedName("creative") CREATIVE,
    @SerializedName("adventure") ADVENTURE,
    @SerializedName("spectator") SPECTATOR
}
