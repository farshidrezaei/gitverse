#[cfg(test)]
mod tests {
    use super::super::lfs::*;

    #[test]
    fn test_parse_lfs_attributes() {
        let sample_attributes = r#"
# LFS tracking rules
*.psd filter=lfs diff=lfs merge=lfs -text
*.zip filter=lfs diff=lfs merge=lfs -text
*.rs text
assets/big.bin filter=lfs diff=lfs merge=lfs -text
"#;
        let patterns = parse_lfs_attributes(sample_attributes);
        assert_eq!(patterns.len(), 3);
        assert_eq!(patterns[0], "*.psd");
        assert_eq!(patterns[1], "*.zip");
        assert_eq!(patterns[2], "assets/big.bin");
    }

    #[test]
    fn test_parse_lfs_ls_files() {
        let sample_output = r#"
4d51a24d8b * design/mockup.psd
c1f2e3d4a5 - build/release.zip
"#;
        let files = parse_lfs_ls_files(sample_output);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].object_identifier, "4d51a24d8b");
        assert_eq!(files[0].file_path, "design/mockup.psd");
        assert_eq!(files[1].object_identifier, "c1f2e3d4a5");
        assert_eq!(files[1].file_path, "build/release.zip");
    }

    #[test]
    fn test_parse_lfs_locks() {
        let sample_output = "design/mockup.psd\tFarshid Rezaei\tID:1024\n";
        let locks = parse_lfs_locks(sample_output);
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].file_path, "design/mockup.psd");
        assert_eq!(locks[0].owner_name, "Farshid Rezaei");
        assert_eq!(locks[0].lock_identifier, "1024");
    }
}
