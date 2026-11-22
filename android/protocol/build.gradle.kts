plugins {
    kotlin("jvm")
}

kotlin {
    jvmToolchain(17)
}

dependencies {
    implementation("org.bouncycastle:bcprov-jdk18on:1.79")
    testImplementation(kotlin("test-junit5"))
    testImplementation("org.json:json:20240303")
}

tasks.test {
    useJUnitPlatform()
}
