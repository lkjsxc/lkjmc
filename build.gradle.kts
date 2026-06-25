import org.gradle.api.tasks.SourceSetContainer
import org.gradle.api.tasks.bundling.Jar
import org.gradle.api.tasks.compile.JavaCompile
import org.gradle.api.tasks.testing.Test

plugins {
    base
}

subprojects {
    group = "com.lkjmc"
    version = "0.0.0"

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
