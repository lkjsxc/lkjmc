plugins {
    `java-library`
}

val docsBundleDir = layout.buildDirectory.dir("generated-resources/docs")
val buildDocsBundle by tasks.registering(Exec::class) {
    val output = docsBundleDir.map { it.file("lkjmc-docs-bundle.json") }
    inputs.files(rootProject.file("README.md"), rootProject.file("AGENTS.md"), rootProject.fileTree("docs") { include("**/*.md") })
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
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.3")
}

tasks.processResources { dependsOn(buildDocsBundle) }

tasks.test {
    useJUnitPlatform()
}
