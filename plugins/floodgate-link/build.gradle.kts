plugins { java }
dependencies {
    implementation(project(":common"))
    compileOnly("org.geysermc.floodgate:api:2.2.5-SNAPSHOT") { isTransitive = false }
    compileOnly("org.geysermc.geyser:common:2.2.1-SNAPSHOT:unshaded") { isTransitive = false }
}
tasks.jar {
    dependsOn(configurations.runtimeClasspath)
    archiveFileName.set("floodgate-lkjmc-database.jar")
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
    from(configurations.runtimeClasspath.get().map { if (it.isDirectory) it else zipTree(it) })
    exclude("META-INF/*.SF", "META-INF/*.RSA", "META-INF/*.DSA", "module-info.class")
}
