plugins {
    `java-library`
}

dependencies {
    implementation(project(":platforms:jvm:common"))
    compileOnly("io.papermc.paper:paper-api:1.21.10-R0.1-SNAPSHOT")
}
