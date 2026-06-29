package com.lkjmc.common.docs;

import com.google.gson.Gson;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;

public final class DocBundle {
    private final Map<String, DocFile> files;

    private DocBundle(Map<String, DocFile> files) {
        this.files = Map.copyOf(files);
    }

    public static DocBundle load(InputStream input) {
        if (input == null) throw new IllegalArgumentException("docs bundle resource is missing");
        var raw = new Gson().fromJson(new InputStreamReader(input, StandardCharsets.UTF_8), RawBundle.class);
        var map = new TreeMap<String, DocFile>();
        for (var file : raw.files) {
            var links = file.links == null ? List.<DocLink>of() : file.links.stream()
                .map(link -> new DocLink(link.text, link.target, link.line)).toList();
            map.put(file.path, new DocFile(file.path, file.title, file.lines, links));
        }
        return new DocBundle(map);
    }

    public List<DocFile> files() {
        return files.values().stream().sorted(Comparator.comparing(DocFile::path)).toList();
    }

    public Optional<DocFile> file(String path) {
        return DocPath.normalize(path).flatMap(value -> Optional.ofNullable(files.get(value)));
    }

    public List<String> children(String dir) {
        var prefix = DocPath.normalize(dir).orElse("");
        if (!prefix.isBlank()) prefix = prefix + "/";
        var children = new java.util.TreeSet<String>();
        for (var path : files.keySet()) {
            if (!path.startsWith(prefix)) continue;
            var rest = path.substring(prefix.length());
            if (rest.isBlank()) continue;
            var slash = rest.indexOf('/');
            children.add(slash < 0 ? rest : rest.substring(0, slash) + "/");
        }
        return List.copyOf(children);
    }

    public List<DocFile> search(String query) {
        var needle = query == null ? "" : query.toLowerCase();
        return files().stream()
            .filter(file -> file.path().toLowerCase().contains(needle) || file.title().toLowerCase().contains(needle)
                || file.lines().stream().anyMatch(line -> line.toLowerCase().contains(needle)))
            .limit(21)
            .toList();
    }

    private static final class RawBundle { List<RawFile> files = List.of(); }
    private static final class RawFile { String path; String title; List<String> lines; List<RawLink> links; }
    private static final class RawLink { String text; String target; int line; }
}
