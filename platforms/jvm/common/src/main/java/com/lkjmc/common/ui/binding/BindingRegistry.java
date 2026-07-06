package com.lkjmc.common.ui.binding;

import java.util.ArrayList;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

public final class BindingRegistry {
    private final Map<String, MenuBinding> bindings;

    public BindingRegistry(Collection<MenuBinding> bindings) {
        var values = new LinkedHashMap<String, MenuBinding>();
        for (var binding : bindings == null ? List.<MenuBinding>of() : bindings) {
            if (values.putIfAbsent(binding.id(), binding) != null) {
                throw new IllegalArgumentException("duplicate menu binding: " + binding.id());
            }
        }
        this.bindings = Map.copyOf(values);
    }

    public static BindingRegistry standard() {
        var all = new ArrayList<MenuBinding>();
        all.add(new ServerListBinding());
        all.addAll(TravelBindings.bindings());
        all.add(new RandomTeleportBinding());
        all.addAll(ClaimBindings.bindings());
        all.addAll(EconomyBindings.bindings());
        all.addAll(SocialBindings.bindings());
        all.addAll(ProfileBindings.bindings());
        all.add(new AdventureBinding());
        all.add(new SettingsBinding());
        all.addAll(AdminBindings.bindings());
        all.addAll(DocsBindings.bindings());
        all.addAll(PickerBindings.bindings());
        return new BindingRegistry(all);
    }

    public static BindingRegistry defaults() {
        return standard();
    }

    public Optional<MenuBinding> find(String id) {
        return Optional.ofNullable(bindings.get(id));
    }

    public MenuBinding require(String id) {
        return find(id).orElseThrow(() -> new IllegalArgumentException("unknown menu binding: " + id));
    }

    public Set<String> keys() {
        return bindings.keySet();
    }
}
