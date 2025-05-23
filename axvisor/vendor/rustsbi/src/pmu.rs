use sbi_spec::binary::SbiRet;

/// Performance Monitoring Unit extension.
///
/// The RISC-V hardware performance counters such as `mcycle`, `minstret`, and `mhpmcounterX` CSRs
/// are accessible as read-only from supervisor-mode using `cycle`, `instret`, and `hpmcounterX` CSRs.
/// The SBI performance monitoring unit (PMU) extension is an interface for supervisor-mode to configure
/// and use the RISC-V hardware performance counters with assistance from the machine-mode (or hypervisor-mode).
/// These hardware performance counters can only be started, stopped, or configured from machine-mode
/// using `mcountinhibit` and `mhpmeventX` CSRs.
/// Due to this, a machine-mode SBI implementation may choose to disallow its SBI PMU extension
/// if `mcountinhibit` CSR is not implemented by the RISC-V platform.
///
/// A RISC-V platform generally supports monitoring of various hardware events using a limited number
/// of hardware performance counters which are up to 64 bits wide.
/// In addition, an SBI implementation can also provide firmware performance counters which can monitor firmware events
/// such as number of misaligned load/store instructions, number of RFENCEs, number of IPIs, etc.
/// The firmware counters are always 64 bits wide.
///
/// The SBI PMU extension provides:
///
/// 1. An interface for supervisor-mode software to discover and configure per-HART hardware/firmware counters
/// 2. A typical perf compatible interface for hardware/firmware performance counters and events
/// 3. Full access to micro-architecture’s raw event encodings
///
/// To define SBI PMU extension calls, we first define important entities `counter_idx`, `event_idx`, and `event_data`.
/// The `counter_idx` is a logical number assigned to each hardware/firmware counter.
/// The `event_idx `represents a hardware (or firmware) event, whereas
/// the `event_data` is 64 bits wide and represents additional configuration (or parameters) for
/// a hardware (or firmware) event.
///
/// The event_idx is a 20 bits wide number encoded as follows:
///
/// ```text
/// event_idx[19:16] = type;
/// event_idx[15:0] = code;
/// ```
pub trait Pmu {
    /// Returns the number of counters (both hardware and firmware).
    ///
    /// The value is returned in `SbiRet.value`; this call always returns `SbiRet::success()`.
    fn num_counters(&self) -> usize;
    /// Get details about the specified counter such as underlying CSR number, width of the counter,
    /// type of counter (hardware/firmware, etc.).
    ///
    /// The `counter_info` returned by this SBI call is encoded as follows:
    ///
    /// ```text
    /// counter_info[11:0] = CSR; // (12bit CSR number)
    /// counter_info[17:12] = Width; // (One less than number of bits in CSR)
    /// counter_info[XLEN-2:18] = Reserved; // Reserved for future use
    /// counter_info[XLEN-1] = Type; // (0 = hardware and 1 = firmware)
    /// ```
    /// If `counter_info.type` == `1` then `counter_info.csr` and `counter_info.width` should be ignored.
    ///
    /// # Return value
    ///
    /// Returns the `counter_info` described above in `SbiRet.value`.
    ///
    /// The possible return error codes returned in `SbiRet.error` are shown in the table below:
    ///
    /// | Return code                 | Description
    /// |:----------------------------|:----------------------------------------------
    /// | `SbiRet::success()`         | `counter_info` read successfully.
    /// | `SbiRet::invalid_param()`   | `counter_idx` points to an invalid counter.
    fn counter_get_info(&self, counter_idx: usize) -> SbiRet;
    /// Find and configure a counter from a set of counters which is not started (or enabled)
    /// and can monitor the specified event.
    ///
    /// # Parameters
    ///
    /// The `counter_idx_base` and `counter_idx_mask` parameters represent the set of counters,
    /// whereas the `event_idx` represents the event to be monitored
    /// and `event_data` represents any additional event configuration.
    ///
    /// The `config_flags` parameter represents additional configuration and filter flags on counters.
    /// The bit definitions of the `config_flags` parameter are shown in the table below:
    ///
    /// | Flag Name                      | Bits         | Description
    /// |:-------------------------------|:-------------|:------------
    /// | `SBI_PMU_CFG_FLAG_SKIP_MATCH`  | `0:0`        | Skip the counter matching
    /// | `SBI_PMU_CFG_FLAG_CLEAR_VALUE` | `1:1`        | Clear (or zero) the counter value in counter configuration
    /// | `SBI_PMU_CFG_FLAG_AUTO_START`  | `2:2`        | Start the counter after configuring a matching counter
    /// | `SBI_PMU_CFG_FLAG_SET_VUINH`   | `3:3`        | Event counting inhibited in VU-mode
    /// | `SBI_PMU_CFG_FLAG_SET_VSINH`   | `4:4`        | Event counting inhibited in VS-mode
    /// | `SBI_PMU_CFG_FLAG_SET_UINH`    | `5:5`        | Event counting inhibited in U-mode
    /// | `SBI_PMU_CFG_FLAG_SET_SINH`    | `6:6`        | Event counting inhibited in S-mode
    /// | `SBI_PMU_CFG_FLAG_SET_MINH`    | `7:7`        | Event counting inhibited in M-mode
    /// | _RESERVED_                     | `8:(XLEN-1)` | _All non-zero values are reserved for future use._
    ///
    /// *NOTE:* When *SBI_PMU_CFG_FLAG_SKIP_MATCH* is set in `config_flags`, the
    /// SBI implementation will unconditionally select the first counter from the
    /// set of counters specified by the `counter_idx_base` and `counter_idx_mask`.
    ///
    /// *NOTE:* The *SBI_PMU_CFG_FLAG_AUTO_START* flag in `config_flags` has no
    /// impact on the value of counter.
    ///
    /// *NOTE:* The `config_flags[3:7]` bits are event filtering hints so these
    /// can be ignored or overridden by the SBI implementation for security concerns
    /// or due to lack of event filtering support in the underlying RISC-V platform.
    ///
    /// # Return value
    ///
    /// Returns the `counter_idx` in `sbiret.value` upon success.
    ///
    /// In case of failure, the possible error codes returned in `sbiret.error` are shown in the table below:
    ///
    /// | Return code               | Description
    /// |:--------------------------|:----------------------------------------------
    /// | `SbiRet::success()`       | counter found and configured successfully.
    /// | `SbiRet::invalid_param()` | the set of counters has at least one invalid counter.
    /// | `SbiRet::not_supported()` | none of the counters can monitor the specified event.
    fn counter_config_matching(
        &self,
        counter_idx_base: usize,
        counter_idx_mask: usize,
        config_flags: usize,
        event_idx: usize,
        event_data: u64,
    ) -> SbiRet;
    /// Start or enable a set of counters on the calling HART with the specified initial value.
    ///
    /// # Parameters
    ///
    /// The `counter_idx_base` and `counter_idx_mask` parameters represent the set of counters.
    /// whereas the `initial_value` parameter specifies the initial value of the counter.
    ///
    /// The bit definitions of the `start_flags` parameter are shown in the table below:
    ///
    /// | Flag Name                      | Bits         | Description
    /// |:-------------------------------|:-------------|:------------
    /// | `SBI_PMU_START_SET_INIT_VALUE` | `0:0`        | Set the value of counters based on the `initial_value` parameter.
    /// | _RESERVED_                     | `1:(XLEN-1)` | _All non-zero values are reserved for future use._
    ///
    /// *NOTE*: When `SBI_PMU_START_SET_INIT_VALUE` is not set in `start_flags`, the value of counter will
    /// not be modified, and event counting will start from the current value of counter.
    ///
    /// # Return value
    ///
    /// The possible return error codes returned in `SbiRet.error` are shown in the table below:
    ///
    /// | Return code                 | Description
    /// |:----------------------------|:----------------------------------------------
    /// | `SbiRet::success()`         | counter started successfully.
    /// | `SbiRet::invalid_param()`   | the set of counters has at least one invalid counter.
    /// | `SbiRet::already_started()` | the set of counters includes at least one counter which is already started.
    fn counter_start(
        &self,
        counter_idx_base: usize,
        counter_idx_mask: usize,
        start_flags: usize,
        initial_value: u64,
    ) -> SbiRet;
    /// Stop or disable a set of counters on the calling HART.
    ///
    /// # Parameters
    ///
    /// The `counter_idx_base` and `counter_idx_mask` parameters represent the set of counters.
    /// The bit definitions of the `stop_flags` parameter are shown in the table below:
    ///
    /// | Flag Name                 | Bits         | Description
    /// |:--------------------------|:-------------|:------------
    /// | `SBI_PMU_STOP_FLAG_RESET` | `0:0`        | Reset the counter to event mapping.
    /// | _RESERVED_                | `1:(XLEN-1)` | *All non-zero values are reserved for future use.*
    ///
    /// # Return value
    ///
    /// The possible return error codes returned in `SbiRet.error` are shown in the table below:
    ///
    /// | Return code                 | Description
    /// |:----------------------------|:----------------------------------------------
    /// | `SbiRet::success()`         | counter stopped successfully.
    /// | `SbiRet::invalid_param()`   | the set of counters has at least one invalid counter.
    /// | `SbiRet::already_stopped()` | the set of counters includes at least one counter which is already stopped.
    fn counter_stop(
        &self,
        counter_idx_base: usize,
        counter_idx_mask: usize,
        stop_flags: usize,
    ) -> SbiRet;
    /// Provide the current value of a firmware counter in `SbiRet.value`.
    ///
    /// On RV32 systems, the `SbiRet.value` will only contain the lower 32 bits from the value of
    /// the firmware counter.
    ///
    /// # Parameters
    ///
    /// This function should be only used to read a firmware counter. It will return an error
    /// when the user provides a hardware counter in `counter_idx` parameter.
    ///
    /// # Return value
    ///
    /// The possible return error codes returned in `SbiRet.error` are shown in the table below:
    ///
    /// | Return code               | Description
    /// |:--------------------------|:----------------------------------------------
    /// | `SbiRet::success()`       | firmware counter read successfully.
    /// | `SbiRet::invalid_param()` | `counter_idx` points to a hardware counter or an invalid counter.
    fn counter_fw_read(&self, counter_idx: usize) -> SbiRet;
    /// Provide the upper 32 bits from the current value of firmware counter in `SbiRet.value`.
    ///
    /// This function always returns zero in `SbiRet.value` for RV64 (or higher) systems.
    ///
    /// # Return value
    ///
    /// The possible return error codes returned in `SbiRet.error` are shown in the table below:
    ///
    /// | Return code               | Description
    /// |:--------------------------|:----------------------------------------------
    /// | `SbiRet::success()`       | firmware counter read successfully.
    /// | `SbiRet::invalid_param()` | `counter_idx` points to a hardware counter or an invalid counter.
    #[inline]
    fn counter_fw_read_hi(&self, counter_idx: usize) -> SbiRet {
        match () {
            #[cfg(not(target_pointer_width = "32"))]
            () => {
                let _ = counter_idx;
                SbiRet::success(0)
            }
            #[cfg(target_pointer_width = "32")]
            () => {
                let _ = counter_idx;
                SbiRet::not_supported()
            }
        }
    }
    /// Function internal to macros. Do not use.
    #[doc(hidden)]
    #[inline]
    fn _rustsbi_probe(&self) -> usize {
        sbi_spec::base::UNAVAILABLE_EXTENSION.wrapping_add(1)
    }
}

impl<T: Pmu> Pmu for &T {
    #[inline]
    fn num_counters(&self) -> usize {
        T::num_counters(self)
    }
    #[inline]
    fn counter_get_info(&self, counter_idx: usize) -> SbiRet {
        T::counter_get_info(self, counter_idx)
    }
    #[inline]
    fn counter_config_matching(
        &self,
        counter_idx_base: usize,
        counter_idx_mask: usize,
        config_flags: usize,
        event_idx: usize,
        event_data: u64,
    ) -> SbiRet {
        T::counter_config_matching(
            self,
            counter_idx_base,
            counter_idx_mask,
            config_flags,
            event_idx,
            event_data,
        )
    }
    #[inline]
    fn counter_start(
        &self,
        counter_idx_base: usize,
        counter_idx_mask: usize,
        start_flags: usize,
        initial_value: u64,
    ) -> SbiRet {
        T::counter_start(
            self,
            counter_idx_base,
            counter_idx_mask,
            start_flags,
            initial_value,
        )
    }
    #[inline]
    fn counter_stop(
        &self,
        counter_idx_base: usize,
        counter_idx_mask: usize,
        stop_flags: usize,
    ) -> SbiRet {
        T::counter_stop(self, counter_idx_base, counter_idx_mask, stop_flags)
    }
    #[inline]
    fn counter_fw_read(&self, counter_idx: usize) -> SbiRet {
        T::counter_fw_read(self, counter_idx)
    }
    #[inline]
    fn counter_fw_read_hi(&self, counter_idx: usize) -> SbiRet {
        T::counter_fw_read_hi(self, counter_idx)
    }
}

impl<T: Pmu> Pmu for Option<T> {
    #[inline]
    fn num_counters(&self) -> usize {
        self.as_ref()
            .map(|inner| T::num_counters(inner))
            .unwrap_or(0)
    }
    #[inline]
    fn counter_get_info(&self, counter_idx: usize) -> SbiRet {
        self.as_ref()
            .map(|inner| T::counter_get_info(inner, counter_idx))
            .unwrap_or(SbiRet::not_supported())
    }
    #[inline]
    fn counter_config_matching(
        &self,
        counter_idx_base: usize,
        counter_idx_mask: usize,
        config_flags: usize,
        event_idx: usize,
        event_data: u64,
    ) -> SbiRet {
        self.as_ref().map_or(SbiRet::not_supported(), |inner| {
            T::counter_config_matching(
                inner,
                counter_idx_base,
                counter_idx_mask,
                config_flags,
                event_idx,
                event_data,
            )
        })
    }
    #[inline]
    fn counter_start(
        &self,
        counter_idx_base: usize,
        counter_idx_mask: usize,
        start_flags: usize,
        initial_value: u64,
    ) -> SbiRet {
        self.as_ref()
            .map(|inner| {
                T::counter_start(
                    inner,
                    counter_idx_base,
                    counter_idx_mask,
                    start_flags,
                    initial_value,
                )
            })
            .unwrap_or(SbiRet::not_supported())
    }
    #[inline]
    fn counter_stop(
        &self,
        counter_idx_base: usize,
        counter_idx_mask: usize,
        stop_flags: usize,
    ) -> SbiRet {
        self.as_ref()
            .map(|inner| T::counter_stop(inner, counter_idx_base, counter_idx_mask, stop_flags))
            .unwrap_or(SbiRet::not_supported())
    }
    #[inline]
    fn counter_fw_read(&self, counter_idx: usize) -> SbiRet {
        self.as_ref()
            .map(|inner| T::counter_fw_read(inner, counter_idx))
            .unwrap_or(SbiRet::not_supported())
    }
    #[inline]
    fn counter_fw_read_hi(&self, counter_idx: usize) -> SbiRet {
        self.as_ref()
            .map(|inner| T::counter_fw_read_hi(inner, counter_idx))
            .unwrap_or(SbiRet::not_supported())
    }
    #[inline]
    fn _rustsbi_probe(&self) -> usize {
        match self {
            Some(_) => sbi_spec::base::UNAVAILABLE_EXTENSION.wrapping_add(1),
            None => sbi_spec::base::UNAVAILABLE_EXTENSION,
        }
    }
}
