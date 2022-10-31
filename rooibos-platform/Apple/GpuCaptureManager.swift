import Metal

@_cdecl("GpuCaptureManager_start")
public func GpuCaptureManager_start(
    captureManager: MTLCaptureManager,
    device: MTLDevice,
    tracefile: UnsafePointer<CChar>
) -> Bool {
    let captureDescriptor = MTLCaptureDescriptor()

    captureDescriptor.captureObject = device
    captureDescriptor.destination   = .gpuTraceDocument
    captureDescriptor.outputURL     = URL(string: String(cString: tracefile))

    do {
        try captureManager.startCapture(with: captureDescriptor)
    }  catch let e {
        print("Failed to capture frame for debug: \(e.localizedDescription)")
        return false
    }

    return true
}

@_cdecl("GpuCaptureManager_stop")
public func GpuCaptureManager_stop(captureManager: MTLCaptureManager) {
    captureManager.stopCapture()
}
