plugins { java }
dependencies {
    implementation(project(":common"))
    compileOnly("io.papermc.paper:paper-api:26.2.build.129-stable")
    compileOnly("com.sk89q.worldedit:worldedit-bukkit:7.4.5")
    compileOnly("com.sk89q.worldguard:worldguard-bukkit:7.0.19")
}
tasks.register<Copy>("apiJars") {
    from(configurations.compileClasspath)
    into(layout.buildDirectory.dir("api"))
}
tasks.jar {
    dependsOn(configurations.runtimeClasspath)
    archiveFileName.set("lkjmc-paper.jar")
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
    from(configurations.runtimeClasspath.get().map { if (it.isDirectory) it else zipTree(it) })
    exclude("META-INF/*.SF", "META-INF/*.RSA", "META-INF/*.DSA", "module-info.class")
}

val checkTeamMenus = tasks.register<JavaExec>("checkTeamMenus") {
    dependsOn(tasks.testClasses)
    classpath = sourceSets["test"].runtimeClasspath
    mainClass.set("com.lkjsxc.lkjmc.paper.TeamMenuPolicyTest")
}
tasks.check { dependsOn(checkTeamMenus) }
