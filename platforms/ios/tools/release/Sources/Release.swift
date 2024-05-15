import ArgumentParser
import CommandLineTools
import Foundation

@main
struct Release: AsyncParsableCommand {
    @Option(help: "The version of the library that is being released.")
    var version: String
    
    @Flag(help: "Prevents the run from pushing anything to GitHub.")
    var localOnly = false
    
    var apiToken = ProcessInfo.processInfo.environment["SWIFT_RELEASE_TOKEN"]!
    
    var sourceRepo = Repository(owner: "matrix-org", name: "matrix-rich-text-editor")
    var packageRepo = Repository(owner: "matrix-org", name: "matrix-rich-text-editor-swift")
    
    var buildDirectory = URL(filePath: #file)
        .deletingLastPathComponent() // Release.swift
        .deletingLastPathComponent() // Sources
        .deletingLastPathComponent() // release
        .deletingLastPathComponent() // tools
        .deletingLastPathComponent() // ios
        .deletingLastPathComponent() // platforms
    
    mutating func run() async throws {
        let package = try clonePackageRepo()
        Zsh.defaultDirectory = buildDirectory
        
        Log.info("Build directory: \(buildDirectory.path())")
        
        let product = try build()
        let (zipFileURL, checksum) = try package.zipBinary(with: product)
        
        try await updatePackage(package, with: product, checksum: checksum)
        try commitAndPush(package, with: product)
        try await package.makeRelease(with: product, uploading: zipFileURL)
    }
    
    func clonePackageRepo() throws -> Package {
        Log.info("Checking out Swift package…")
        
        let packageDirectory = buildDirectory.appending(component: packageRepo.name)
        let git = try Git.clone(repository: URL(string: "git@github.com:\(packageRepo.owner)/\(packageRepo.name)")!, directory: packageDirectory)
        try git.fetch()
        try git.checkout(branch: "main")
        
        return Package(repository: packageRepo, directory: packageDirectory, apiToken: apiToken, urlSession: localOnly ? .releaseMock : .shared)
    }
    
    func build() throws -> BuildProduct {
        let git = Git(directory: buildDirectory)
        let commitHash = try git.commitHash
        let branch = try git.branchName
        
        Log.info("Building \(branch) at \(commitHash)")
        
        try Zsh.run(command: "make ios")
        
        return BuildProduct(sourceRepo: sourceRepo,
                            version: version,
                            commitHash: commitHash,
                            branch: branch,
                            directory: buildDirectory.appending(path: "platforms/ios/lib/WysiwygComposer/"),
                            frameworkName: "WysiwygComposerFFI.xcframework")
    }
    
    func updatePackage(_ package: Package, with product: BuildProduct, checksum: String) async throws {
        Log.info("Copying sources")
        let source = product.directory.appending(component: "Sources", directoryHint: .isDirectory).path()
        let destination = package.directory.appending(component: "Sources", directoryHint: .isDirectory).path()
        try Zsh.run(command: "rsync -a --delete '\(source)' '\(destination)'")
        
        try await package.updateManifest(with: product, checksum: checksum)
    }
    
    func commitAndPush(_ package: Package, with product: BuildProduct) throws {
        Log.info("Pushing changes")
        
        let git = Git(directory: package.directory)
        try git.add(files: ".")
        try git.commit(message: "Bump to version \(product.version) (\(product.sourceRepo.name)/\(product.branch) \(product.commitHash))")
        
        guard !localOnly else {
            Log.info("Skipping push for --local-only")
            return
        }
        
        try git.push()
    }
}
