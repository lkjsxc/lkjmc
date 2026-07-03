plugins {
    `java-library`
}

sourceSets {
    main { resources.srcDir(rootProject.file("contracts")) }
}

dependencies {
    api("com.google.code.gson:gson:2.11.0")
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.3")
}

tasks.test {
    useJUnitPlatform()
}
