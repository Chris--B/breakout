import Metal

@_cdecl("GpuCapture_start")
public func GpuCapture_start(
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

@_cdecl("GpuCapture_stop")
public func GpuCapture_stop(captureManager: MTLCaptureManager) {
    captureManager.stopCapture()
}
