// LilynkShared holds what the app AND the widget extension both need — the stored-host model,
// the settings-key names, the App-Group constant, the deep-link grammar, and the Live Activity
// attributes — in a module that links neither the Rust core nor the presentation layer.
//
// Re-export it so every existing consumer of LilynkKit (`import LilynkKit`) keeps seeing
// `StoredHost`, `DefaultsKey`, `lilynkDefaultMgmtPort`, `DeepLink`, etc. with no call-site churn.
// (Files INSIDE LilynkKit still `import LilynkShared` explicitly — Swift imports are
// file-scoped; the re-export only reaches downstream modules.)
@_exported import LilynkShared
