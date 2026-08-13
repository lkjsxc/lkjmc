import java.security.MessageDigest

plugins {
    `java-library`
}

val checkedBindings = rootProject.file("platforms/jvm/common/src/generated/java")
val candidateBindings = layout.buildDirectory.dir("binding-check")
val checkedMenu = rootProject.file("platforms/jvm/common/src/generated/resources/lkjmc-menu-bundle.json")
val candidateMenu = layout.buildDirectory.file("menu-check/lkjmc-menu-bundle.json")
val buildInfoRoot = layout.buildDirectory.dir("generated/sources/lkjmcBuildInfo/java")
val buildInfoSource = buildInfoRoot.map { it.file("com/lkjmc/common/LkjmcBuildInfo.java") }
val buildVersion = rootProject.extra["lkjmcVersion"] as String
val buildLicense = rootProject.extra["lkjmcLicense"] as String
val buildCommit = rootProject.extra["lkjmcBuildCommit"] as String
val buildDirty = rootProject.extra["lkjmcBuildDirty"] as String
val generateBuildInfo by tasks.registering {
    inputs.property("version", buildVersion)
    inputs.property("license", buildLicense)
    inputs.property("commit", buildCommit)
    inputs.property("dirty", buildDirty)
    outputs.file(buildInfoSource)
    doLast {
        val output = buildInfoSource.get().asFile
        output.parentFile.mkdirs()
        output.writeText("""package com.lkjmc.common;

public final class LkjmcBuildInfo {
    public static final String VERSION = "$buildVersion";
    public static final String LICENSE = "$buildLicense";
    public static final String COMMIT = "$buildCommit";
    public static final String DIRTY = "$buildDirty";

    private LkjmcBuildInfo() {}

    public static void main(String[] arguments) {
        System.out.println(VERSION + "\t" + LICENSE + "\t" + COMMIT + "\t" + DIRTY);
    }
}
""")
    }
}
sourceSets.main {
    java.srcDir(checkedBindings)
    java.srcDir(buildInfoRoot)
    resources.srcDir(rootProject.file("platforms/jvm/common/src/generated/resources"))
}

val compileMenuBundle by tasks.registering(Exec::class) {
    inputs.file(rootProject.file("scripts/compile-menu-bundle.py"))
    inputs.file(rootProject.file("contracts/menus/README.json"))
    inputs.files(rootProject.fileTree("contracts/menus") { include("*.json") })
    inputs.files(rootProject.file("config/locales/en.json"), rootProject.file("config/locales/ja.json"))
    outputs.file(candidateMenu)
    doFirst { candidateMenu.get().asFile.parentFile.mkdirs() }
    commandLine("python3", rootProject.file("scripts/compile-menu-bundle.py"), candidateMenu.get().asFile)
}

val verifyMenuBundle by tasks.registering {
    dependsOn(compileMenuBundle)
    inputs.file(checkedMenu)
    doLast {
        if (!checkedMenu.isFile || !checkedMenu.readBytes().contentEquals(candidateMenu.get().asFile.readBytes()))
            throw GradleException("stale menu bundle; run updateMenuBundle")
    }
}

val updateMenuBundle by tasks.registering(Exec::class) {
    doFirst { checkedMenu.parentFile.mkdirs() }
    commandLine("python3", rootProject.file("scripts/compile-menu-bundle.py"), checkedMenu)
}

val generateJvmBindings by tasks.registering(Exec::class) {
    val output = candidateBindings.get().asFile
    inputs.files(rootProject.file("platforms/jvm/contracts/sync.json"),
        rootProject.file("platforms/jvm/contracts/consumption.json"),
        rootProject.file("contracts/commands/README.json"),
        rootProject.fileTree("contracts/commands") { include("*.json") })
    inputs.files(rootProject.fileTree("platforms/jvm") { include("*.py") },
        rootProject.file("crates/lkjmc-daemon/src/transport/sync.rs"),
        rootProject.file("crates/lkjmc-store/src/sync/payload.rs"),
        rootProject.file("crates/lkjmc-core/src/profile_envelope.rs"))
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

tasks.compileJava { dependsOn(generateBuildInfo, verifyJvmBindings, verifyMenuBundle) }
tasks.check { dependsOn(verifyJvmBindings, verifyMenuBundle) }

tasks.processResources {
    dependsOn(verifyMenuBundle)
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

val menuContractMutations by tasks.registering(Test::class) {
    dependsOn(tasks.testClasses)
    testClassesDirs = sourceSets.test.get().output.classesDirs
    classpath = sourceSets.test.get().runtimeClasspath
    useJUnitPlatform()
    filter { includeTestsMatching("com.lkjmc.common.menu.MenuContractTest.rejectsGenericMutationBody") }
}

tasks.register<JavaExec>("syncHarness") {
    dependsOn(tasks.testClasses)
    classpath = sourceSets.test.get().runtimeClasspath
    mainClass.set("com.lkjmc.common.sync.SyncHarness")
    args(providers.gradleProperty("syncProbe").getOrElse("all"))
    workingDir(rootProject.projectDir)
}
