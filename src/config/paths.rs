/// Represents different file and directory paths
pub enum Paths {
	/// The Archive dir holds packages.
	/// Default dir `/var/cache/apt/archives/`
	Archive,
	/// The Lists dir hold package lists from `update` command.
	/// Default dir `/var/lib/apt/lists/`
	Lists,
	/// The main Source List.
	/// Default file `/etc/apt/sources.list`
	SourceList,
	/// The Sources parts directory
	/// Default dir `/etc/apt/sources.list.d/`
	SourceParts,
	/// Nala Sources file is generated from the `fetch` command.
	/// Default file `/etc/apt/sources.list.d/nala-sources.list`
	NalaSources,

	History,
}
