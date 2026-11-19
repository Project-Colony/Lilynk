---
title: Forgot your Password?
description: Read or reset the web console's login password on a Linux, SteamOS or Windows host.
---

Read the web console's login password on the host, or reset it. This is only the console login; a
client that can't connect needs [Pairing](/docs/pairing).

| Host | Password file | Restart the console |
|---|---|---|
| Linux packages, NixOS | `~/.config/lilynk/web-password` | `systemctl --user restart lilynk-web` |
| SteamOS | `~/.config/lilynk/web.env` | `systemctl --user restart lilynk-web` |
| Windows | `%ProgramData%\lilynk\web-password` | `lilynk-host service restart`, elevated |

## Read it

The file holds the password in clear text until your first sign-in. After that the console keeps
only a salted hash, and you [reset it](#reset-it) instead.

```sh
sed -n 's/^LILYNK_UI_PASSWORD=//p' ~/.config/lilynk/web-password   # Linux
sed -n 's/^LILYNK_UI_PASSWORD=//p' ~/.config/lilynk/web.env        # SteamOS
```

On Windows, from an elevated PowerShell (the file is readable by Administrators only):

```powershell
lilynk-host web password
```

## Reset it

Write a `LILYNK_UI_PASSWORD=<new-password>` line into the file and restart the console. The next
sign-in hashes it and signs every other session out.

```sh
printf 'LILYNK_UI_PASSWORD=%s\n' 'new-password' > ~/.config/lilynk/web-password
chmod 600 ~/.config/lilynk/web-password
systemctl --user restart lilynk-web
```

- SteamOS: edit `web.env` rather than overwriting it, because it also holds the session secret.
- Windows, from an elevated PowerShell:

  ```powershell
  notepad "$env:ProgramData\lilynk\web-password"   # add LILYNK_UI_PASSWORD=<new-password>
  lilynk-host service restart
  ```

On the Linux packages you can instead delete `web-password` and restart the console. It generates a
new password to [read](#read-it).

## The password is right, and sign-in still fails

- **Too many sign-in attempts. Try again …**: after five wrong tries, a device waits up to five
  minutes. Wait, or restart the console to clear it.
- **Wrong password.** for every password, and other console pages answer `auth not configured`: the
  file has no password line. Add one as in [Reset it](#reset-it).
