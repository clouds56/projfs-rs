ProjFS
===========
[![Chrono on crates.io][cratesio-image]](https://crates.io/crates/projfs)
[![Chrono on docs.rs][docsrs-image]](https://docs.rs/projfs)

See [example](examples/regfs.rs) for more information

[cratesio-image]: https://img.shields.io/crates/v/projfs.svg
[docsrs-image]: https://docs.rs/projfs/badge.svg

Get Start
-----
One should create a struct `MyProjFS` that implements `Sync`, `ProjFSDirEnum` and `ProjFSRead` in order to get a instance.
```rust
// create root dir to be projected
std::fs::create_dir("root_dir").ok();
// create a virtualization instance of MyProjFS
// this function returned immediately, and you would like to hold the instance during the projection
let instance = start_proj_virtualization("root_dir", Box::new(MyProjFS::new())).unwrap();
std::thread::sleep(std::time::Duration::from_secs(std::u64::MAX));
// once the instance dropped, the projection stopped
drop(instance)
```

Features
-----
See also mircosoft guide [here](https://docs.microsoft.com/en-us/windows/win32/projfs/projfs-programming-guide)
Now we could provide [callback functions](https://docs.microsoft.com/en-us/windows/win32/projfs/projfs-callback-functions)

- [ ] `PRJ_CANCEL_COMMAND_CB`
- [x] `PRJ_END_DIRECTORY_ENUMERATION_CB`
- [x] `PRJ_GET_DIRECTORY_ENUMERATION_CB`
- [x] `PRJ_GET_FILE_DATA_CB` (via `ProjFSRead::read`)
- [x] `PRJ_GET_PLACEHOLDER_INFO_CB` (via `ProjFSRead::get_metadata`)
- [ ] `PRJ_NOTIFICATION_CB`
- [ ] `PRJ_QUERY_FILE_NAME_CB`
- [x] `PRJ_START_DIRECTORY_ENUMERATION_CB`

Callback series `PRJ_*_DIRECTORY_ENUMERATION_CB` would be generate by `ProjFSDirEnum::dir_iter` and `ProjFSDirEnum::dir_iter_cache`.

Note
-----
Make sure Projected File System is enabled on your machine
```powershell
Enable-WindowsOptionalFeature -Online -FeatureName Client-ProjFS -NoRestart
```
