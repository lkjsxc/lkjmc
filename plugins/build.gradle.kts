import groovy.json.JsonOutput
import groovy.json.JsonSlurper
import java.security.MessageDigest

plugins { java }
allprojects {
    group = "com.lkjsxc.lkjmc"
    version = "0.1.0"
    repositories {
        mavenCentral()
        maven("https://repo.papermc.io/repository/maven-public/")
        maven("https://maven.enginehub.org/repo/")
        maven("https://repo.opencollab.dev/main/")
    }
}
subprojects {
    apply(plugin = "java")
    dependencyLocking { lockAllConfigurations() }
    extensions.configure<JavaPluginExtension> { toolchain.languageVersion.set(JavaLanguageVersion.of(25)) }
    tasks.withType<JavaCompile>().configureEach { options.encoding = "UTF-8" }
    tasks.withType<Jar>().configureEach { isPreserveFileTimestamps = false; isReproducibleFileOrder = true }
    // Gradle 9.8's verification metadata writer fails on duplicate snapshot module
    // identities. Lock versions and verify the actual compiler/runtime JAR bytes.
    val verifyDependencyBytes = tasks.register("verifyDependencyBytes") {
        doLast {
            val artifacts = listOf("compileClasspath", "runtimeClasspath", "annotationProcessor")
                .flatMap { configurations.getByName(it).resolvedConfiguration.resolvedArtifacts }
                .filter { it.moduleVersion.id.group != "com.lkjsxc.lkjmc" }
            val actual = artifacts.associate { artifact ->
                "${artifact.moduleVersion.id}:${artifact.file.name}" to
                    MessageDigest.getInstance("SHA-256").digest(artifact.file.readBytes()).joinToString("") { "%02x".format(it) }
            }.toSortedMap()
            val manifest = file("dependency-sha256.json")
            if (providers.gradleProperty("updateDependencyHashes").orNull == "true") {
                manifest.writeText(JsonOutput.prettyPrint(JsonOutput.toJson(actual)) + "\n")
            } else {
                check(manifest.exists()) { "Missing dependency checksum manifest" }
                check(JsonSlurper().parse(manifest) == actual) { "Dependency JAR bytes changed; inspect the version and artifact source before updating checksums" }
            }
        }
    }
    tasks.withType<JavaCompile>().configureEach { dependsOn(verifyDependencyBytes) }
}
