package org.projectcolony.lilynk

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/**
 * Pure JVM test of the launcher-file halves in Links.kt: the name a title saves under, and the
 * link a frontend's file hands back. Run: `./gradlew -PexcludeScreenshots :app:testDebugUnitTest`.
 */
class LinksTest {
    private val url = "lilynk://connect/abc?launch=steam:570"

    @Test
    fun aTitleSavesUnderANameEverySdCardTakes() {
        assertEquals("Halo Reach.lilynk", launcherFileName("Halo: Reach"))
        assertEquals("AC DC Live.lilynk", launcherFileName("AC/DC  Live?"))
        assertEquals("Lilynk.lilynk", launcherFileName(" ... "))
    }

    @Test
    fun theFilesFirstNonBlankLineIsTheLink() {
        assertEquals(url, launcherFileLink("$url\n"))
        assertEquals(url, launcherFileLink("﻿\r\n  $url  \r\nignored"))
        assertEquals("ly://connect/abc", launcherFileLink("ly://connect/abc"))
    }

    @Test
    fun aFileWithNoLinkOfOursYieldsNone() {
        assertNull(launcherFileLink(""))
        assertNull(launcherFileLink("https://example.com/\n$url"))
        assertNull(launcherFileLink("[Desktop Entry]"))
    }

    @Test
    fun aLinkTheRoutersWillRefuseStillReachesThemToSayWhy() {
        assertEquals("lilynk://pair/abc", launcherFileLink("lilynk://pair/abc"))
    }
}
