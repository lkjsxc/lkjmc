plugins {
    `java-library`
}

sourceSets {
    main { resources.srcDir(rootProject.file("contracts")) }
}

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
}

tasks.test {
    useJUnitPlatform()
}
