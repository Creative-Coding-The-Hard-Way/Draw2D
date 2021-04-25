initSidebarItems({"enum":[["AdaptiveFormat","Specifies the `FormatFunction` and decides if coloring should be used."],["Age","The age after which a log file rotation will be triggered, when `Criterion::Age` is chosen."],["Cleanup","Defines the strategy for handling older log files."],["Criterion","Criterion when to rotate the log file."],["Duplicate","Used to control which messages are to be duplicated to stderr, when `log_to_file()` is used."],["FlexiLoggerError","Describes errors in the initialization of `flexi_logger`."],["Level","Re-exports from log crate"],["LevelFilter","Re-exports from log crate"],["LogTarget","Describes the default log target."],["Naming","The naming convention for rotated log files."]],"fn":[["colored_default_format","A colored version of the logline-formatter `default_format` that produces log lines like  ERROR [my_prog::some_submodule] File not found"],["colored_detailed_format","A colored version of the logline-formatter `detailed_format`."],["colored_opt_format","A colored version of the logline-formatter `opt_format`."],["colored_with_thread","A colored version of the logline-formatter `with_thread`."],["default_format","A logline-formatter that produces log lines like  `INFO [my_prog::some_submodule] Task successfully read from conf.json`"],["detailed_format","A logline-formatter that produces log lines like  `[2016-01-13 15:25:01.640870 +01:00] INFO [foo::bar] src/foo/bar.rs:26: Task successfully read from conf.json`  i.e. with timestamp, module path and file location."],["opt_format","A logline-formatter that produces log lines with timestamp and file location, like  `[2016-01-13 15:25:01.640870 +01:00] INFO [src/foo/bar:26] Task successfully read from conf.json` "],["style","Helper function that is used in the provided coloring format functions to apply colors based on the log level and the effective color palette."],["with_thread","A logline-formatter that produces log lines like  `[2016-01-13 15:25:01.640870 +01:00] T[taskreader] INFO [src/foo/bar:26] Task successfully read from conf.json`  i.e. with timestamp, thread name and file location."]],"mod":[["code_examples","Here are some examples for the `flexi_logger` initialization."],["writers","Contains the trait `LogWriter` for extending `flexi_logger` with additional log writers, and two concrete implementations for writing to files (`FileLogWriter`) or to the syslog (`SyslogWriter`). You can also use your own implementations of `LogWriter`."]],"struct":[["DeferredNow","Deferred timestamp creation."],["LogSpecBuilder","Builder for `LogSpecification`."],["LogSpecification","Immutable struct that defines which loglines are to be written, based on the module, the log level, and the text."],["Logger","The entry-point for using `flexi_logger`."],["LoggerHandle","Allows reconfiguring the logger programmatically."],["ModuleFilter","Defines which loglevel filter to use for the specified module."],["Record","Re-exports from log crate"]],"type":[["FormatFunction","Function type for Format functions."],["ReconfigurationHandle","For backwards compatibility."]]});