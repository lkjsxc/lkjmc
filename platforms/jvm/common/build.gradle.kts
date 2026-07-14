import java.security.MessageDigest

plugins {
    `java-library`
}

val checkedBindings = rootProject.file("platforms/jvm/common/src/generated/java")
val candidateBindings = layout.buildDirectory.dir("binding-check")
sourceSets.main { java.srcDir(checkedBindings) }

val generateJvmBindings by tasks.registering(Exec::class) {
    val output = candidateBindings.get().asFile
    inputs.files(rootProject.file("platforms/jvm/contracts/sync.json"),
        rootProject.file("platforms/jvm/contracts/consumption.json"),
        rootProject.file("contracts/commands/README.json"),
        rootProject.fileTree("contracts/commands") { include("*.json") })
    inputs.file(rootProject.file("platforms/jvm/generate-bindings.py"))
    outputs.dir(output)
    doFirst { delete(output); output.mkdirs() }
    commandLine("python3", rootProject.file("platforms/jvm/generate-bindings.py"),
        "--root", rootProject.projectDir, "--output", output)
}

val verifyJvmBindings by tasks.registering {
    dependsOn(generateJvmBindings)
    inputs.dir(checkedBindings)
    doLast {
        fun digest(file: File): String = MessageDigest.getInstance("SHA-256")
            .digest(file.readBytes()).joinToString("") { "%02x".format(it) }
        val expected = checkedBindings.walkTopDown().filter { it.isFile }
            .associate { it.relativeTo(checkedBindings).path to digest(it) }
        val actualRoot = candidateBindings.get().asFile
        val actual = actualRoot.walkTopDown().filter { it.isFile }
            .associate { it.relativeTo(actualRoot).path to digest(it) }
        if (expected != actual) throw GradleException("stale JVM bindings; run updateJvmBindings")
    }
}

tasks.register<Exec>("updateJvmBindings") {
    doFirst { delete(checkedBindings); checkedBindings.mkdirs() }
    commandLine("python3", rootProject.file("platforms/jvm/generate-bindings.py"),
        "--root", rootProject.projectDir, "--output", checkedBindings)
}

tasks.compileJava { dependsOn(verifyJvmBindings) }
tasks.check { dependsOn(verifyJvmBindings) }

tasks.processResources {
    from(rootProject.file("config/locales")) {
        include("*.json")
        into("locales")
    }
}

dependencies {
    api("com.google.code.gson:gson:2.11.0")
    compileOnly("net.kyori:adventure-api:4.25.0")
    compileOnly("net.kyori:adventure-text-minimessage:4.25.0")
    testImplementation("net.kyori:adventure-api:4.25.0")
    testImplementation("net.kyori:adventure-text-minimessage:4.25.0")
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.3")
    testImplementation("org.postgresql:postgresql:42.7.7")
}

tasks.test {
    useJUnitPlatform()
}

tasks.register<JavaExec>("syncHarness") {
    dependsOn(tasks.testClasses)
    classpath = sourceSets.test.get().runtimeClasspath
    mainClass.set("com.lkjmc.common.sync.SyncHarness")
    args(providers.gradleProperty("syncProbe").getOrElse("all"))
    workingDir(rootProject.projectDir)
}
