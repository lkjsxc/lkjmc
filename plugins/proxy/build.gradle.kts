plugins { java }
dependencies {
    implementation(project(":common"))
    compileOnly("com.velocitypowered:velocity-api:4.2.1-SNAPSHOT")
    annotationProcessor("com.velocitypowered:velocity-api:4.2.1-SNAPSHOT")
    // The adapter uses Floodgate identity/link interfaces, not its runtime or old
    // transitive Geyser distribution. Floodgate supplies these classes at runtime.
    compileOnly("org.geysermc.floodgate:api:2.2.5-SNAPSHOT") { isTransitive = false }
}
tasks.register<Copy>("apiJars") {
    from(configurations.compileClasspath)
    into(layout.buildDirectory.dir("api"))
}
tasks.jar {
    dependsOn(configurations.runtimeClasspath)
    archiveFileName.set("lkjmc-velocity.jar")
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
    from(configurations.runtimeClasspath.get().map { if (it.isDirectory) it else zipTree(it) })
    exclude("META-INF/*.SF", "META-INF/*.RSA", "META-INF/*.DSA", "module-info.class")
}
