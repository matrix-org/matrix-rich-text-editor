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
        let packageDirectory = try clonePackageRepo()
        let package = Package(repository: packageRepo, directory: packageDirectory, apiToken: apiToken, urlSession: localOnly ? .releaseMock : .shared)
        Zsh.defaultDirectory = buildDirectory
        
        Log.info("Build directory: \(buildDirectory.path())")
        
        let product = try build()
        let (zipFileURL, checksum) = try package.zipBinary(with: product)
        
        try await updatePackage(package, with: product, checksum: checksum)
        try commitAndPush(package, with: product)
        try await package.makeRelease(with: product, uploading: zipFileURL)
    }
    
    func clonePackageRepo() throws -> URL {
        Log.info("Checking out Swift package…")
        let packageDirectory = buildDirectory.appending(component: "matrix-rich-text-editor-swift")
        if !FileManager.default.fileExists(atPath: packageDirectory.path()) {
            try Zsh.run(command: "git clone git@github.com:\(packageRepo.owner)/\(packageRepo.name) \(packageDirectory.path())")
        }
        try Zsh.run(command: "git fetch && git checkout main", directory: packageDirectory)
        return packageDirectory
    }
    
    func build() throws -> BuildProduct {
        let commitHash = try Zsh.run(command: "git rev-parse HEAD")!.trimmingCharacters(in: .whitespacesAndNewlines)
        let branch = try Zsh.run(command: "git rev-parse --abbrev-ref HEAD")!.trimmingCharacters(in: .whitespacesAndNewlines)
        
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
        try Zsh.run(command: "git add .", directory: package.directory)
        try Zsh.run(command: "git commit -m 'Bump to version \(product.version) (\(product.sourceRepo.name)/\(product.branch) \(product.commitHash))'", directory: package.directory)
        
        guard !localOnly else {
            Log.info("Skipping push for --local-only")
            return
        }
        
        try Zsh.run(command: "git push", directory: package.directory)
    }
}
