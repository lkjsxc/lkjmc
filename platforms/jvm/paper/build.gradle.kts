plugins {
    `java-library`
}

val docsBundleDir = layout.buildDirectory.dir("generated-resources/docs")
val buildDocsBundle by tasks.registering(Exec::class) {
    val output = docsBundleDir.map { it.file("lkjmc-docs-bundle.json") }
    inputs.file(rootProject.file("scripts/build-docs-bundle.py"))
    inputs.files(rootProject.file("README.md"), rootProject.file("AGENTS.md"), rootProject.fileTree("docs") {
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

tasks.processResources { dependsOn(buildDocsBundle) }

tasks.test {
    useJUnitPlatform()
}

val jvmProbes by tasks.registering(JavaExec::class) {
    dependsOn(tasks.testClasses, tasks.named("shadowJar"),
        project(":platforms:jvm:velocity").tasks.named("shadowJar"))
    classpath = sourceSets.test.get().runtimeClasspath
    mainClass.set("com.lkjmc.paper.harness.JvmProbeRunner")
    doFirst {
        args(layout.buildDirectory.file("libs/paper-0.0.0-all.jar").get().asFile.absolutePath,
            project(":platforms:jvm:velocity").layout.buildDirectory
                .file("libs/velocity-0.0.0-all.jar").get().asFile.absolutePath)
    }
}

tasks.check { dependsOn(jvmProbes) }
