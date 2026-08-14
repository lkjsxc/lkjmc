plugins {
    `java-library`
}

val docsBundleDir = layout.buildDirectory.dir("generated-resources/docs")
val buildDocsBundle by tasks.registering(Exec::class) {
    val output = docsBundleDir.map { it.file("lkjmc-docs-bundle.json") }
    inputs.files(rootProject.file("scripts/build-docs-bundle.py"),
        rootProject.file("contracts/docs-player-corpus.json"))
    inputs.files(rootProject.file("README.md"), rootProject.fileTree("docs") {
        include("**/*.md")
        exclude("**/archive/**")
    })
    outputs.file(output)
    commandLine("python3", rootProject.file("scripts/build-docs-bundle.py"), output.get().asFile)
}

sourceSets {
    main { resources.srcDir(docsBundleDir) }
}

dependencies {
    implementation(project(":platforms:jvm:common"))
    compileOnly("io.papermc.paper:paper-api:1.21.10-R0.1-SNAPSHOT")
    testImplementation("io.papermc.paper:paper-api:1.21.10-R0.1-SNAPSHOT")
    testImplementation(project(":platforms:jvm:velocity"))
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.3")
}

tasks.processResources {
    dependsOn(buildDocsBundle)
    inputs.property("pluginVersion", project.version)
    filesMatching("plugin.yml") { expand("version" to project.version) }
}

tasks.test {
    useJUnitPlatform()
}

val jvmProbes by tasks.registering(JavaExec::class) {
    dependsOn(tasks.testClasses, tasks.named("shadowJar"),
        project(":platforms:jvm:velocity").tasks.named("shadowJar"))
    classpath = sourceSets.test.get().runtimeClasspath
    mainClass.set("com.lkjmc.paper.harness.JvmProbeRunner")
    providers.gradleProperty("jvmProbe").orNull?.let {
        val names = setOf("scheduler-blocks-zero", "typed-bindings-all", "folia-ownership-pass",
            "velocity-routing-pass", "transfer-outcomes-pass", "workflow-ack-pass",
            "plugin-shutdown-pass", "duplicate-jvm-paths-absent")
        require(it in names) { "unknown JVM probe: $it" }
        systemProperty("lkjmc.jvm.probe", it)
    }
    doFirst {
        args(layout.buildDirectory.file("libs/paper-all.jar").get().asFile.absolutePath,
            project(":platforms:jvm:velocity").layout.buildDirectory
                .file("libs/velocity-all.jar").get().asFile.absolutePath)
    }
}

val menuCheckerMutations by tasks.registering {
    dependsOn(project(":platforms:jvm:common").tasks.named("menuContractMutations"))
}

val menuProbeNames = setOf("all-routes-selected-engine", "golden-state-matrix",
    "no-unintended-close", "local-menu-daemon-independent", "locale-parity-quality",
    "protocol-menu-pass", "old-menu-engine-absent")

val menuProbes by tasks.registering(JavaExec::class) {
    dependsOn(tasks.testClasses, tasks.named("shadowJar"))
    classpath = sourceSets.test.get().runtimeClasspath
    mainClass.set("com.lkjmc.paper.harness.MenuProbeRunner")
    providers.gradleProperty("menuProbe").orNull?.let {
        require(it in menuProbeNames) { "unknown menu probe: $it" }
        systemProperty("lkjmc.menu.probe", it)
    }
    doFirst { args(layout.buildDirectory.file("libs/paper-all.jar").get().asFile.absolutePath) }
}

val updateMenuGoldens by tasks.registering(JavaExec::class) {
    dependsOn(tasks.testClasses, tasks.named("shadowJar"))
    classpath = sourceSets.test.get().runtimeClasspath
    mainClass.set("com.lkjmc.paper.harness.MenuProbeRunner")
    systemProperty("lkjmc.menu.goldens.write",
        rootProject.file("platforms/jvm/paper/src/test/resources/menu/menu-goldens.json"))
    doFirst { args(layout.buildDirectory.file("libs/paper-all.jar").get().asFile.absolutePath) }
}

tasks.check { dependsOn(jvmProbes, menuProbes, menuCheckerMutations) }
