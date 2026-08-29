import org.gradle.api.tasks.SourceSetContainer
import org.gradle.api.tasks.bundling.Jar
import org.gradle.api.tasks.compile.JavaCompile
import org.gradle.api.tasks.testing.Test

plugins {
    base
}

fun workspacePackageValue(name: String): String {
    var inWorkspacePackage = false
    for (raw in rootProject.file("Cargo.toml").readLines()) {
        val line = raw.trim()
        if (line.startsWith("[") && line.endsWith("]")) {
            inWorkspacePackage = line == "[workspace.package]"
        } else if (inWorkspacePackage) {
            val match = Regex("^${Regex.escape(name)}\\s*=\\s*\"([^\"]+)\"$").matchEntire(line)
            if (match != null) return match.groupValues[1]
        }
    }
    throw GradleException("missing workspace.package $name")
}

fun gitResult(vararg arguments: String): Pair<Int, String>? = try {
    val process = ProcessBuilder(listOf("git", "-C", rootProject.projectDir.absolutePath) + arguments)
        .redirectErrorStream(true)
        .start()
    val output = process.inputStream.bufferedReader().use { it.readText().trim() }
    process.waitFor() to output
} catch (_: Exception) {
    null
}

val releaseVersion = workspacePackageValue("version")
if (!Regex("[0-9]+\\.[0-9]+\\.[0-9]+(?:-[0-9A-Za-z.-]+)?").matches(releaseVersion)) {
    throw GradleException("workspace version is not a supported release version")
}
val releaseLicense = workspacePackageValue("license")
if (releaseLicense != "Apache-2.0") {
    throw GradleException("workspace license must match the Apache-2.0 root LICENSE")
}

val suppliedCommit = providers.environmentVariable("LKJMC_SOURCE_COMMIT").orNull
    ?.takeUnless { it.isEmpty() }
val buildNonce = providers.environmentVariable("LKJMC_BUILD_NONCE").orNull
    ?.takeUnless { it.isEmpty() }
if (suppliedCommit != null && !Regex("[0-9a-f]{40}").matches(suppliedCommit)) {
    throw GradleException("LKJMC_SOURCE_COMMIT must be 40 lowercase hexadecimal characters")
}
if (buildNonce != null && !Regex("[0-9a-f]{32}").matches(buildNonce)) {
    throw GradleException("LKJMC_BUILD_NONCE must be 32 lowercase hexadecimal characters")
}
val gitProbe = gitResult("rev-parse", "--is-inside-work-tree")
val hasGit = gitProbe != null && gitProbe.first == 0 && gitProbe.second == "true"
if (suppliedCommit != null && !hasGit) {
    throw GradleException("LKJMC_SOURCE_COMMIT requires a Git checkout")
}
if (suppliedCommit != null && buildNonce == null) {
    throw GradleException("LKJMC_SOURCE_COMMIT requires LKJMC_BUILD_NONCE")
}
if (suppliedCommit == null && buildNonce != null) {
    throw GradleException("LKJMC_BUILD_NONCE requires LKJMC_SOURCE_COMMIT")
}
val observedCommit = if (hasGit) {
    val result = gitResult("rev-parse", "HEAD")
    if (result == null || result.first != 0 || !Regex("[0-9a-f]{40}").matches(result.second)) {
        throw GradleException("cannot resolve Git HEAD for JVM build identity")
    }
    result.second
} else null
if (suppliedCommit != null && suppliedCommit != observedCommit) {
    throw GradleException("LKJMC_SOURCE_COMMIT differs from Git HEAD")
}
if (suppliedCommit != null) {
    val result = gitResult("status", "--porcelain=v1", "--untracked-files=normal")
    if (result == null || result.first != 0) {
        throw GradleException("cannot inspect Git worktree for JVM build identity")
    }
    if (result.second.isNotEmpty()) {
        throw GradleException("LKJMC_SOURCE_COMMIT requires a clean worktree")
    }
}
val buildCommit = suppliedCommit ?: observedCommit ?: "unknown"
val buildDirty = if (buildNonce != null) "false" else "unknown"

extra["lkjmcVersion"] = releaseVersion
extra["lkjmcLicense"] = releaseLicense
extra["lkjmcBuildCommit"] = buildCommit
extra["lkjmcBuildDirty"] = buildDirty

subprojects {
    group = "com.lkjmc"
    version = releaseVersion

    plugins.withType<JavaPlugin> {
        extensions.configure<JavaPluginExtension> {
            toolchain.languageVersion.set(JavaLanguageVersion.of(21))
        }
        tasks.withType<JavaCompile>().configureEach {
            options.encoding = "UTF-8"
            options.release.set(21)
        }
        tasks.withType<Test>().configureEach {
            testLogging.events("failed")
        }
        tasks.withType<Jar>().configureEach {
            archiveVersion.set("")
            isPreserveFileTimestamps = false
            isReproducibleFileOrder = true
            manifest.attributes(mapOf(
                "Implementation-Title" to project.name,
                "Implementation-Version" to releaseVersion,
                "Bundle-License" to releaseLicense,
                "LKJMC-Build-Commit" to buildCommit,
                "LKJMC-Build-Dirty" to buildDirty,
            ))
        }
        val sourceSets = extensions.getByType<SourceSetContainer>()
        val runtimeClasspath = configurations.getByName("runtimeClasspath")
        tasks.register<Jar>("shadowJar") {
            archiveClassifier.set("all")
            duplicatesStrategy = DuplicatesStrategy.EXCLUDE
            dependsOn(runtimeClasspath)
            from(sourceSets.getByName("main").output)
            from(runtimeClasspath.map { if (it.isDirectory) it else zipTree(it) })
        }
    }
}
