use std::time::{Duration, SystemTime};

use axum::{extract::State, response::IntoResponse};
use jiff::SignedDuration;
use sysinfo::{Components, Disks, System};

use crate::{consts::GIGABYTE, core::CorePtr, response, rlock, util};

pub async fn main(State(core): State<CorePtr>) -> impl IntoResponse {
	let programs = rlock!(core.programs);

	let mut system = System::new_all();
	system.refresh_all();

	let disks = Disks::new_with_refreshed_list();
	let components = Components::new_with_refreshed_list();

	let available_space = disks.iter().map(|disk| disk.available_space()).sum::<u64>();
	let total_space = disks.iter().map(|disk| disk.total_space()).sum::<u64>();

	let mut response = String::from("Server:\n");
	response.push_str(&format!("  Version: {}\n", env!("CARGO_PKG_VERSION")));
	response.push_str(&format!(
		"  Uptime: {:#}\n",
		SignedDuration::from_secs(core.start_time.elapsed().unwrap_or_default().as_secs() as i64)
	));
	response.push_str(&format!("  Start Time: {:#}\n", util::timestamp(Some(core.start_time))));
	response.push_str(&format!(
		"  Running Programs: {}/{} ({})\n",
		programs.iter().filter(|(_, program)| program.is_active()).count(),
		programs.len(),
		programs
			.iter()
			.filter(|(_, program)| program.is_active())
			.map(|(name, _)| name.to_owned())
			.collect::<Vec<_>>()
			.join(", ")
	));
	response.push('\n');

	response.push_str("System:\n");
	response.push_str(&format!(
		"  Version: {}\n",
		System::long_os_version()
			.or_else(System::os_version)
			.unwrap_or_else(|| String::from("N/A"))
	));
	response.push_str(&format!(
		"  Uptime: {:#}\n",
		SignedDuration::from_secs(System::uptime() as i64)
	));
	response.push_str(&format!(
		"  Boot Time: {:#}\n",
		util::timestamp(Some(SystemTime::UNIX_EPOCH + Duration::from_secs(System::boot_time())))
	));
	response.push_str(&format!("  Processes: {}\n", system.processes().len()));
	response.push('\n');

	response.push_str("  CPU Load:\n");
	response.push_str(
		&system
			.cpus()
			.iter()
			.map(|cpu| {
				format!(
					"    {}: {:.2}% ({:.2} GHz)\n",
					cpu.name(),
					cpu.cpu_usage(),
					cpu.frequency() as f64 / 1000.0
				)
			})
			.collect::<Vec<_>>()
			.join(""),
	);
	response.push_str(&format!(
		"    Total: {:.2}% ({:.2} GHz)\n",
		system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / system.cpus().len() as f32,
		system
			.cpus()
			.iter()
			.map(|cpu| cpu.frequency() as f64 / 1000.0)
			.sum::<f64>()
			/ system.cpus().len() as f64
	));
	response.push('\n');

	response.push_str("  RAM Usage:\n");
	response.push_str(&format!(
		"    Memory: {:.2} / {:.2} GB\n",
		system.used_memory() as f64 / GIGABYTE,
		system.total_memory() as f64 / GIGABYTE
	));
	response.push_str(&format!(
		"    Swap: {:.2} / {:.2} GB\n",
		system.used_swap() as f64 / GIGABYTE,
		system.total_swap() as f64 / GIGABYTE
	));
	response.push('\n');

	response.push_str("  Disk Usage:\n");
	response.push_str(
		&disks
			.iter()
			.map(|disk| {
				format!(
					"    {}: {:.2} / {:.2} GB\n",
					disk.name().to_string_lossy(),
					(disk.total_space() - disk.available_space()) as f64 / GIGABYTE,
					disk.total_space() as f64 / GIGABYTE
				)
			})
			.collect::<Vec<_>>()
			.join(""),
	);
	response.push_str(&format!(
		"    Total: {:.2} / {:.2} GB\n",
		(total_space - available_space) as f64 / GIGABYTE,
		total_space as f64 / GIGABYTE
	));
	response.push('\n');

	response.push_str("  Temperatures:\n");
	response.push_str(
		&components
			.iter()
			.map(|component| {
				format!(
					"    {}: {:.2} °C\n",
					component.label(),
					component.temperature().unwrap_or(0.0)
				)
			})
			.collect::<Vec<_>>()
			.join(""),
	);
	response.push('\n');

	response!(OK, response)
}
