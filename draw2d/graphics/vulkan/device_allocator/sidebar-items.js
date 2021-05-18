initSidebarItems({"enum":[["MemUnit","Units of measurement for memory."]],"fn":[["build_standard_allocator","Build the standard allocator implementation."]],"struct":[["Allocation","A single allocated piece of device memory."],["ConsoleMarkdownReport","Build a human-friendly markdown report which is printed directly to the console."],["ForcedOffsetAllocator","An allocator which forces all allocations to have a fixed offset."],["MetricsAllocator","A device allocator decorator which records the number of allocations and other metrics. A summary of results is printed when the allocator is destroyed."],["PageAllocator","Decorate an allocator such that all allocation requests are rounded up to the nearest page."],["PassthroughAllocator","An allocator implementation which just directly allocates a new piece of device memory on each call."],["PoolAllocator",""],["SharedRefAllocator","A device allocator implementation which represents a shared reference to an underlying allocator implementation."],["SizeSelector",""],["Suballocator","A suballocator can divvy up a single allocation into multiple non-overlapping allocations."],["TypeIndexAllocator","The type index allocator creates a separate memory allocator for each memory type index, then dispatches allocations and frees."]],"trait":[["DeviceAllocator","The external device memory allocation interface. This is the api used by applications to allocate and free memory on the gpu."]]});