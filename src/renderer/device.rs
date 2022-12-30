use ash::{vk, extensions::khr};

pub struct QueueFamily {
    pub index: u32,
    pub flags: vk::QueueFlags,
    pub queues: Vec<vk::Queue>
}

pub struct Device {
    pub physical: vk::PhysicalDevice,
    pub logical: ash::Device,
    pub graphics_family: QueueFamily
}

impl Device {
    pub fn new(instance: &ash::Instance, layer_names: &Vec<*const i8>) -> Self {
        let extension_names = vec![khr::Swapchain::name().as_ptr()];
        let physical = Self::pick_physical(&instance);

        let mut graphics_family = Self::pick_queue_family(&instance, physical);

        let queue_priorities = [1.0];

        let queue_infos = vec![
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(graphics_family.index)
                .queue_priorities(&queue_priorities)
                .build()
        ];

        let logical_device_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layer_names);

        let logical = unsafe {
            instance.create_device(physical, &logical_device_info, None).unwrap()
        };

        graphics_family.queues.push(unsafe {
            logical.get_device_queue(graphics_family.index, 0)
        });

        Self {
            physical,
            logical,
            graphics_family
        }
    }

    fn pick_physical(instance: &ash::Instance) -> vk::PhysicalDevice {
        let pds = unsafe { 
            instance.enumerate_physical_devices().unwrap()
        };
        for pd in pds {
            let props = unsafe { 
                instance.get_physical_device_properties(pd)
            };

            if props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                return pd;
            }
        }

        panic!("No physical devices found!");
    }

    fn pick_queue_family(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> QueueFamily {
        let qfps = unsafe {
            instance.get_physical_device_queue_family_properties(physical_device)
        };

        for (i, qfp) in qfps.iter().enumerate() {
            if qfp.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                {
                return QueueFamily {
                    index: i as u32,
                    flags: qfp.queue_flags,
                    queues: vec![]
                }
            }
        }

        panic!("No graphics queue family found!");
    }

    pub unsafe fn cleanup(&mut self) {
        self.logical.destroy_device(None);
    }
}
