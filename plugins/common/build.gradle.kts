plugins { `java-library` }
dependencies { api("com.google.code.gson:gson:2.13.2") }
// The same reviewed catalog is packaged into Paper and Velocity by common.jar.
tasks.processResources { from("../../locales") { into("locales") } }
