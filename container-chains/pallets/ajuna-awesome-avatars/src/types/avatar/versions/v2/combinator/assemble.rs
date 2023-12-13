use super::*;
use sp_std::ops::Not;

impl<T: Config> AvatarCombinator<T> {
    pub(super) fn assemble_avatars(
        input_leader: WrappedForgeItem<T>,
        input_sacrifices: Vec<WrappedForgeItem<T>>,
        hash_provider: &mut HashProvider<T, 32>,
    ) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
        let (matching_sacrifices, non_matching): (Vec<_>, Vec<_>) =
            input_sacrifices.into_iter().partition(|(_, sacrifice)| {
                sacrifice.same_assemble_version(&input_leader.1)
                    || sacrifice.has_full_type(ItemType::Special, SpecialItemType::ToolBox)
            });

        let leader_progress_array = input_leader.1.get_progress();

        let ((leader_id, mut input_leader), matching_sacrifices) = Self::match_avatars(
            input_leader,
            matching_sacrifices,
            MATCH_ALGO_START_RARITY.as_byte(),
            hash_provider,
        );

        let (additionals, non_additionals): (Vec<_>, Vec<_>) = matching_sacrifices
            .into_iter()
            .chain(non_matching)
            .partition(|(_, sacrifice)| {
                sacrifice
                    .same_full_and_class_types(&input_leader)
                    .not()
                    .then(|| {
                        DnaUtils::is_progress_match(
                            leader_progress_array,
                            sacrifice.get_progress(),
                            MATCH_ALGO_START_RARITY.as_byte(),
                        )
                        .is_some()
                    })
                    .unwrap_or(false)
            });

        let progress_rarity = RarityTier::from_byte(DnaUtils::lowest_progress_byte(
            &input_leader.get_progress(),
            ByteType::High,
        ));

        if input_leader.has_full_type(ItemType::Equippable, EquippableItemType::ArmorBase)
            && input_leader.get_rarity() < progress_rarity
        {
            // Add a component to the base armor, only first component will be added
            if let Some((_, armor_component)) = additionals.iter().find(|(_, sacrifice)| {
                sacrifice.has_type(ItemType::Equippable)
                    && !sacrifice.has_subtype(EquippableItemType::ArmorBase)
            }) {
                input_leader.set_spec(
                    SpecIdx::Byte1,
                    input_leader.get_spec::<u8>(SpecIdx::Byte1)
                        | armor_component.get_spec::<u8>(SpecIdx::Byte1),
                );
            }
        }

        input_leader.set_rarity(progress_rarity);

        let output_vec: Vec<ForgeOutput<T>> = additionals
            .into_iter()
            .map(|(sacrifice_id, _)| ForgeOutput::Consumed(sacrifice_id))
            .chain(
                non_additionals
                    .into_iter()
                    .map(|(sacrifice_id, _)| ForgeOutput::Consumed(sacrifice_id)),
            )
            .collect();

        Ok((
            LeaderForgeOutput::Forged((leader_id, input_leader.unwrap()), 0),
            output_vec,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::*;

    #[test]
    fn test_assemble_single_base() {
        ExtBuilder::default().build().execute_with(|| {
            let forge_hash = [
                0x3F, 0x83, 0x25, 0x3B, 0xA9, 0x24, 0xF2, 0xF6, 0xB5, 0xA9, 0x37, 0x15, 0x25, 0x2C,
                0x04, 0xFC, 0xEC, 0x45, 0xC1, 0x4D, 0x86, 0xE7, 0x96, 0xE5, 0x20, 0x5D, 0x8B, 0x39,
                0xB0, 0x54, 0xFB, 0x62,
            ];
            let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

            let hash_base = [
                [
                    0xE7, 0x46, 0x00, 0xE4, 0xE8, 0x78, 0x12, 0xC4, 0xCA, 0x86, 0x53, 0x7F, 0x36,
                    0x1B, 0x64, 0xA0, 0xC3, 0x6B, 0x5C, 0x5F, 0x13, 0x40, 0xBC, 0xC6, 0x97, 0x12,
                    0x25, 0x48, 0xC5, 0xD9, 0x05, 0xC3,
                ],
                [
                    0x3B, 0x06, 0x56, 0x4C, 0x0C, 0x96, 0x6F, 0x41, 0x28, 0x85, 0x40, 0xEC, 0x53,
                    0xAB, 0xF4, 0xCE, 0xCE, 0x6C, 0x60, 0x81, 0xBE, 0xBC, 0xCF, 0x82, 0xBD, 0x70,
                    0x61, 0x14, 0xA2, 0x5E, 0x1A, 0x13,
                ],
                [
                    0x81, 0xA7, 0xCD, 0x5A, 0x36, 0x51, 0xB8, 0xB6, 0xE8, 0x9F, 0x6C, 0xE4, 0xE3,
                    0x52, 0x15, 0xD0, 0xEB, 0xF5, 0x25, 0x97, 0xA7, 0xD2, 0xE4, 0xC0, 0xDC, 0x7C,
                    0xF3, 0x6F, 0xE0, 0xB3, 0x88, 0x76,
                ],
                [
                    0x1A, 0x1C, 0x41, 0x48, 0x0B, 0x96, 0xF9, 0xDC, 0xDA, 0x7A, 0x40, 0x28, 0x99,
                    0x86, 0x58, 0xC3, 0x6A, 0xD4, 0x7C, 0x66, 0x58, 0xD1, 0x9C, 0x8E, 0x81, 0xCF,
                    0xE5, 0x78, 0x70, 0x68, 0x12, 0x0D,
                ],
                [
                    0x84, 0x78, 0x9A, 0x96, 0x77, 0xA2, 0xCE, 0xC3, 0x0E, 0x3C, 0x29, 0x20, 0x33,
                    0x01, 0x67, 0x9C, 0xF8, 0x4D, 0x03, 0x36, 0x80, 0xB5, 0x37, 0x43, 0x6C, 0x71,
                    0xAA, 0xA9, 0x3D, 0x9F, 0x8C, 0xB8,
                ],
            ];

            let mut hash_generators = [
                HashProvider::new_with_bytes([
                    0x40, 0xBC, 0xC6, 0x97, 0x12, 0x25, 0x48, 0xC5, 0xD9, 0x05, 0xC3, 0x40, 0xBC,
                    0xC6, 0x97, 0x12, 0x25, 0x48, 0xC5, 0xD9, 0x05, 0xC3, 0x40, 0xBC, 0xC6, 0x97,
                    0x12, 0x25, 0x48, 0xC5, 0xD9, 0x05,
                ]),
                HashProvider::new_with_bytes([
                    0xBC, 0xCF, 0x82, 0xBD, 0x70, 0x61, 0x14, 0xA2, 0x5E, 0x1A, 0x13, 0xBC, 0xCF,
                    0x82, 0xBD, 0x70, 0x61, 0x14, 0xA2, 0x5E, 0x1A, 0x13, 0xBC, 0xCF, 0x82, 0xBD,
                    0x70, 0x61, 0x14, 0xA2, 0x5E, 0x1A,
                ]),
                HashProvider::new_with_bytes([
                    0xD2, 0xE4, 0xC0, 0xDC, 0x7C, 0xF3, 0x6F, 0xE0, 0xB3, 0x88, 0x76, 0xD2, 0xE4,
                    0xC0, 0xDC, 0x7C, 0xF3, 0x6F, 0xE0, 0xB3, 0x88, 0x76, 0xD2, 0xE4, 0xC0, 0xDC,
                    0x7C, 0xF3, 0x6F, 0xE0, 0xB3, 0x88,
                ]),
                HashProvider::new_with_bytes([
                    0xD1, 0x9C, 0x8E, 0x81, 0xCF, 0xE5, 0x78, 0x70, 0x68, 0x12, 0x0D, 0xD1, 0x9C,
                    0x8E, 0x81, 0xCF, 0xE5, 0x78, 0x70, 0x68, 0x12, 0x0D, 0xD1, 0x9C, 0x8E, 0x81,
                    0xCF, 0xE5, 0x78, 0x70, 0x68, 0x12,
                ]),
                HashProvider::new_with_bytes([
                    0xB5, 0x37, 0x43, 0x6C, 0x71, 0xAA, 0xA9, 0x3D, 0x9F, 0x8C, 0xB8, 0xB5, 0x37,
                    0x43, 0x6C, 0x71, 0xAA, 0xA9, 0x3D, 0x9F, 0x8C, 0xB8, 0xB5, 0x37, 0x43, 0x6C,
                    0x71, 0xAA, 0xA9, 0x3D, 0x9F, 0x8C,
                ]),
            ];

            let mut armor_component_set = hash_base
                .into_iter()
                .enumerate()
                .map(|(i, hash)| {
                    create_random_armor_component(
                        hash,
                        &ALICE,
                        &PetType::FoxishDude,
                        &SlotType::Head,
                        &RarityTier::Common,
                        &[EquippableItemType::ArmorBase],
                        &(ColorType::Null, ColorType::Null),
                        &Force::Null,
                        i as SoulCount,
                        &mut hash_generators[i],
                    )
                })
                .collect::<Vec<_>>();

            let total_soul_points = armor_component_set
                .iter()
                .map(|(_, avatar)| avatar.get_souls())
                .sum::<SoulCount>();
            assert_eq!(total_soul_points, 10);

            let armor_component_sacrifices = armor_component_set.split_off(1);
            let leader_armor_component = armor_component_set.pop().unwrap();

            let expected_progress_array = [
                0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13,
            ];

            assert_eq!(
                leader_armor_component.1.get_progress(),
                expected_progress_array
            );

            let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::assemble_avatars(
                leader_armor_component,
                armor_component_sacrifices,
                &mut hash_provider,
            )
            .expect("Should succeed in forging");

            assert_eq!(sacrifice_output.len(), 4);
            assert_eq!(
                sacrifice_output
                    .iter()
                    .filter(|output| is_consumed(output))
                    .count(),
                4
            );
            assert_eq!(
                sacrifice_output
                    .iter()
                    .filter(|output| is_forged(output))
                    .count(),
                0
            );

            if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
                let wrapped = WrappedAvatar::new(avatar);
                assert_eq!(wrapped.get_souls(), 10);

                let leader_progress_array = wrapped.get_progress();
                let expected_leader_progress_array = [
                    0x14, 0x22, 0x20, 0x11, 0x20, 0x21, 0x20, 0x15, 0x11, 0x25, 0x23,
                ];
                assert_eq!(leader_progress_array, expected_leader_progress_array);
            } else {
                panic!("LeaderForgeOutput should have been Forged!")
            }
        });
    }

    #[test]
    fn test_assemble_single_base_with_component_1() {
        ExtBuilder::default().build().execute_with(|| {
            let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

            let hash_base = [
                [
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1,
                ],
                [
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2,
                ],
                [
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA3,
                ],
                [
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA4,
                ],
                [
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA5,
                ],
            ];

            let mut hash_generators = [
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA3, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA3, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA4, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA4, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA5, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA5, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
            ];

            let progress_arrays = [
                [
                    0x21, 0x10, 0x25, 0x23, 0x20, 0x23, 0x21, 0x22, 0x22, 0x22, 0x24,
                ],
                [
                    0x21, 0x15, 0x22, 0x15, 0x13, 0x12, 0x12, 0x10, 0x13, 0x10, 0x15,
                ],
                [
                    0x12, 0x13, 0x11, 0x12, 0x12, 0x20, 0x12, 0x13, 0x13, 0x12, 0x15,
                ],
                [
                    0x11, 0x11, 0x25, 0x24, 0x14, 0x23, 0x13, 0x12, 0x12, 0x12, 0x12,
                ],
                [
                    0x11, 0x11, 0x25, 0x24, 0x14, 0x23, 0x13, 0x12, 0x25, 0x12, 0x12,
                ],
            ];

            let mut armor_component_set = [
                EquippableItemType::ArmorBase,
                EquippableItemType::ArmorBase,
                EquippableItemType::ArmorBase,
                EquippableItemType::ArmorBase,
                EquippableItemType::ArmorComponent1,
            ]
            .into_iter()
            .zip(hash_base)
            .zip(progress_arrays)
            .enumerate()
            .map(|(i, ((equip_type, hash), progress_array))| {
                let (id, mut avatar) = create_random_armor_component(
                    hash,
                    &ALICE,
                    &PetType::FoxishDude,
                    &SlotType::Head,
                    &RarityTier::Common,
                    &[equip_type],
                    &(ColorType::Null, ColorType::Null),
                    &Force::Null,
                    i as SoulCount,
                    &mut hash_generators[i],
                );
                avatar.set_progress(progress_array);
                (id, avatar)
            })
            .collect::<Vec<_>>();

            let total_soul_points = armor_component_set
                .iter()
                .map(|(_, avatar)| avatar.get_souls())
                .sum::<SoulCount>();
            assert_eq!(total_soul_points, 10);

            let armor_component_sacrifices = armor_component_set.split_off(1);
            let leader_armor_component = armor_component_set.pop().unwrap();

            let expected_progress_array = [
                0x21, 0x10, 0x25, 0x23, 0x20, 0x23, 0x21, 0x22, 0x22, 0x22, 0x24,
            ];
            assert_eq!(
                leader_armor_component.1.get_progress(),
                expected_progress_array
            );

            let pre_assemble = DnaUtils::bits_to_enums::<EquippableItemType>(
                leader_armor_component.1.get_spec::<u8>(SpecIdx::Byte1) as u32,
            );
            assert_eq!(pre_assemble.len(), 1);
            assert_eq!(pre_assemble[0], EquippableItemType::ArmorBase);

            let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::assemble_avatars(
                leader_armor_component,
                armor_component_sacrifices,
                &mut hash_provider,
            )
            .expect("Should succeed in forging");

            assert_eq!(sacrifice_output.len(), 4);
            assert_eq!(
                sacrifice_output
                    .iter()
                    .filter(|output| is_consumed(output))
                    .count(),
                4
            );
            assert_eq!(
                sacrifice_output
                    .iter()
                    .filter(|output| is_forged(output))
                    .count(),
                0
            );

            if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
                let wrapped = WrappedAvatar::new(avatar);
                assert_eq!(wrapped.get_souls(), 10);

                let post_assemble = DnaUtils::bits_to_enums::<EquippableItemType>(
                    wrapped.get_spec::<u8>(SpecIdx::Byte1) as u32,
                );
                assert_eq!(post_assemble.len(), 2);
                assert_eq!(post_assemble[0], EquippableItemType::ArmorBase);
                assert_eq!(post_assemble[1], EquippableItemType::ArmorComponent1);

                let leader_progress_array = wrapped.get_progress();
                let expected_leader_progress_array = [
                    0x21, 0x20, 0x25, 0x23, 0x20, 0x23, 0x21, 0x22, 0x22, 0x22, 0x24,
                ];
                assert_eq!(leader_progress_array, expected_leader_progress_array);
            } else {
                panic!("LeaderForgeOutput should have been Forged!")
            }
        });
    }

    #[test]
    fn test_assemble_single_base_with_component_2() {
        ExtBuilder::default().build().execute_with(|| {
            let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

            let mut hash_generators = [
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA3, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA3, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA4, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA4, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA5, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA5, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
            ];

            let progress_arrays = [
                [
                    0x21, 0x10, 0x25, 0x23, 0x20, 0x23, 0x21, 0x22, 0x22, 0x22, 0x24,
                ],
                [
                    0x21, 0x15, 0x22, 0x15, 0x13, 0x12, 0x12, 0x10, 0x13, 0x10, 0x15,
                ],
                [
                    0x12, 0x13, 0x11, 0x12, 0x12, 0x20, 0x12, 0x13, 0x13, 0x12, 0x15,
                ],
                [
                    0x11, 0x11, 0x25, 0x24, 0x14, 0x23, 0x13, 0x12, 0x12, 0x12, 0x12,
                ],
                [
                    0x11, 0x11, 0x25, 0x24, 0x14, 0x23, 0x13, 0x12, 0x25, 0x12, 0x12,
                ],
            ];

            let mut armor_component_set = [
                EquippableItemType::ArmorBase,
                EquippableItemType::ArmorComponent1,
                EquippableItemType::ArmorComponent1,
                EquippableItemType::ArmorComponent2,
                EquippableItemType::ArmorComponent1,
            ]
            .into_iter()
            .zip(progress_arrays)
            .enumerate()
            .map(|(i, (equip_type, progress_array))| {
                let (id, mut avatar) = create_random_armor_component(
                    [0; 32],
                    &ALICE,
                    &PetType::FoxishDude,
                    &SlotType::Head,
                    &RarityTier::Common,
                    &[equip_type],
                    &(ColorType::Null, ColorType::Null),
                    &Force::Null,
                    i as SoulCount,
                    &mut hash_generators[i],
                );
                avatar.set_progress(progress_array);
                (id, avatar)
            })
            .collect::<Vec<_>>();

            let armor_component_sacrifices = armor_component_set.split_off(1);
            let leader_armor_component = armor_component_set.pop().unwrap();

            let expected_dna = [
                0x41, 0x12, 0x01, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x10, 0x25, 0x23, 0x20, 0x23, 0x21,
                0x22, 0x22, 0x22, 0x24,
            ];
            assert_eq!(leader_armor_component.1.get_dna().as_slice(), &expected_dna);

            let (leader_output, _) = AvatarCombinator::<Test>::assemble_avatars(
                leader_armor_component,
                armor_component_sacrifices,
                &mut hash_provider,
            )
            .expect("Should succeed in forging");

            if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
                let wrapped = WrappedAvatar::new(avatar);
                assert_eq!(wrapped.get_souls(), 10);

                let post_assemble = DnaUtils::bits_to_enums::<EquippableItemType>(
                    wrapped.get_spec::<u8>(SpecIdx::Byte1) as u32,
                );
                assert_eq!(post_assemble.len(), 2);
                assert_eq!(post_assemble[0], EquippableItemType::ArmorBase);
                assert_eq!(post_assemble[1], EquippableItemType::ArmorComponent2);

                let leader_progress_array = wrapped.get_progress();
                let expected_leader_progress_array = [
                    0x21, 0x20, 0x25, 0x23, 0x20, 0x23, 0x21, 0x22, 0x22, 0x22, 0x24,
                ];
                assert_eq!(leader_progress_array, expected_leader_progress_array);

                let expected_dna = [
                    0x41, 0x12, 0x02, 0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x20, 0x25, 0x23, 0x20,
                    0x23, 0x21, 0x22, 0x22, 0x22, 0x24,
                ];
                assert_eq!(wrapped.get_dna().as_slice(), &expected_dna);
            } else {
                panic!("LeaderForgeOutput should have been Forged!")
            }
        });
    }

    #[test]
    fn test_assemble_single_base_with_component_3() {
        ExtBuilder::default().build().execute_with(|| {
            let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

            let mut hash_generators = [
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA3, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA3, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA4, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA4, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA5, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA5, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
            ];

            let progress_arrays = [
                [
                    0x21, 0x10, 0x25, 0x23, 0x20, 0x23, 0x21, 0x22, 0x22, 0x22, 0x24,
                ],
                [
                    0x21, 0x15, 0x22, 0x15, 0x13, 0x12, 0x12, 0x10, 0x13, 0x10, 0x15,
                ],
                [
                    0x12, 0x13, 0x11, 0x12, 0x12, 0x20, 0x12, 0x13, 0x13, 0x12, 0x15,
                ],
                [
                    0x11, 0x11, 0x25, 0x24, 0x14, 0x23, 0x13, 0x12, 0x12, 0x12, 0x12,
                ],
                [
                    0x11, 0x11, 0x25, 0x24, 0x14, 0x23, 0x13, 0x12, 0x25, 0x12, 0x12,
                ],
            ];

            let mut armor_component_set = [
                EquippableItemType::ArmorBase,
                EquippableItemType::ArmorComponent1,
                EquippableItemType::ArmorComponent1,
                EquippableItemType::ArmorComponent3,
                EquippableItemType::ArmorComponent2,
            ]
            .into_iter()
            .zip(progress_arrays)
            .enumerate()
            .map(|(i, (equip_type, progress_array))| {
                let (id, mut avatar) = create_random_armor_component(
                    [0; 32],
                    &ALICE,
                    &PetType::FoxishDude,
                    &SlotType::Head,
                    &RarityTier::Common,
                    &[equip_type],
                    &(ColorType::Null, ColorType::Null),
                    &Force::Null,
                    i as SoulCount,
                    &mut hash_generators[i],
                );
                avatar.set_progress(progress_array);
                (id, avatar)
            })
            .collect::<Vec<_>>();

            let armor_component_sacrifices = armor_component_set.split_off(1);
            let leader_armor_component = armor_component_set.pop().unwrap();

            let expected_dna = [
                0x41, 0x12, 0x01, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x10, 0x25, 0x23, 0x20, 0x23, 0x21,
                0x22, 0x22, 0x22, 0x24,
            ];
            assert_eq!(leader_armor_component.1.get_dna().as_slice(), &expected_dna);

            let (leader_output, _) = AvatarCombinator::<Test>::assemble_avatars(
                leader_armor_component,
                armor_component_sacrifices,
                &mut hash_provider,
            )
            .expect("Should succeed in forging");

            if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
                let wrapped = WrappedAvatar::new(avatar);
                assert_eq!(wrapped.get_souls(), 10);

                let post_assemble = DnaUtils::bits_to_enums::<EquippableItemType>(
                    wrapped.get_spec::<u8>(SpecIdx::Byte1) as u32,
                );
                assert_eq!(post_assemble.len(), 2);
                assert_eq!(post_assemble[0], EquippableItemType::ArmorBase);
                assert_eq!(post_assemble[1], EquippableItemType::ArmorComponent3);

                let leader_progress_array = wrapped.get_progress();
                let expected_leader_progress_array = [
                    0x21, 0x20, 0x25, 0x23, 0x20, 0x23, 0x21, 0x22, 0x22, 0x22, 0x24,
                ];
                assert_eq!(leader_progress_array, expected_leader_progress_array);

                let expected_dna = [
                    0x41, 0x12, 0x02, 0x01, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x20, 0x25, 0x23, 0x20,
                    0x23, 0x21, 0x22, 0x22, 0x22, 0x24,
                ];
                assert_eq!(wrapped.get_dna().as_slice(), &expected_dna);
            } else {
                panic!("LeaderForgeOutput should have been Forged!")
            }
        });
    }

    #[test]
    fn test_assemble_single_base_with_component_4() {
        ExtBuilder::default().build().execute_with(|| {
            let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

            let mut hash_generators = [
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA3, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA3, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA4, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA4, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA5, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA5, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
            ];

            let progress_arrays = [
                [
                    0x31, 0x20, 0x35, 0x33, 0x30, 0x33, 0x31, 0x32, 0x32, 0x32, 0x34,
                ],
                [
                    0x31, 0x25, 0x32, 0x25, 0x23, 0x22, 0x22, 0x20, 0x23, 0x20, 0x25,
                ],
                [
                    0x22, 0x23, 0x21, 0x22, 0x22, 0x30, 0x22, 0x23, 0x23, 0x22, 0x25,
                ],
                [
                    0x21, 0x21, 0x35, 0x34, 0x24, 0x33, 0x23, 0x22, 0x22, 0x22, 0x22,
                ],
                [
                    0x21, 0x21, 0x35, 0x34, 0x24, 0x33, 0x23, 0x22, 0x35, 0x22, 0x22,
                ],
            ];

            let mut armor_component_set = [
                vec![
                    EquippableItemType::ArmorBase,
                    EquippableItemType::ArmorComponent1,
                ],
                vec![EquippableItemType::ArmorComponent1],
                vec![EquippableItemType::ArmorComponent1],
                vec![EquippableItemType::ArmorComponent3],
                vec![EquippableItemType::ArmorComponent2],
            ]
            .into_iter()
            .zip(progress_arrays)
            .enumerate()
            .map(|(i, (equip_type, progress_array))| {
                let (id, mut avatar) = create_random_armor_component(
                    [0; 32],
                    &ALICE,
                    &PetType::FoxishDude,
                    &SlotType::Head,
                    &RarityTier::Uncommon,
                    equip_type.as_slice(),
                    &(ColorType::Null, ColorType::Null),
                    &Force::Null,
                    i as SoulCount,
                    &mut hash_generators[i],
                );
                avatar.set_progress(progress_array);
                (id, avatar)
            })
            .collect::<Vec<_>>();

            let armor_component_sacrifices = armor_component_set.split_off(1);
            let leader_armor_component = armor_component_set.pop().unwrap();

            let expected_dna = [
                0x41, 0x12, 0x02, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x31, 0x20, 0x35, 0x33, 0x30, 0x33, 0x31,
                0x32, 0x32, 0x32, 0x34,
            ];
            assert_eq!(leader_armor_component.1.get_dna().as_slice(), &expected_dna);

            let (leader_output, _) = AvatarCombinator::<Test>::assemble_avatars(
                leader_armor_component,
                armor_component_sacrifices,
                &mut hash_provider,
            )
            .expect("Should succeed in forging");

            if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
                let wrapped = WrappedAvatar::new(avatar);
                assert_eq!(wrapped.get_souls(), 10);

                let post_assemble = DnaUtils::bits_to_enums::<EquippableItemType>(
                    wrapped.get_spec::<u8>(SpecIdx::Byte1) as u32,
                );
                assert_eq!(post_assemble.len(), 3);
                assert_eq!(post_assemble[0], EquippableItemType::ArmorBase);
                assert_eq!(post_assemble[1], EquippableItemType::ArmorComponent1);
                assert_eq!(post_assemble[2], EquippableItemType::ArmorComponent3);

                let leader_progress_array = wrapped.get_progress();
                let expected_leader_progress_array = [
                    0x31, 0x30, 0x35, 0x33, 0x30, 0x33, 0x31, 0x32, 0x32, 0x32, 0x34,
                ];
                assert_eq!(leader_progress_array, expected_leader_progress_array);

                let expected_dna = [
                    0x41, 0x12, 0x03, 0x01, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x31, 0x30, 0x35, 0x33, 0x30,
                    0x33, 0x31, 0x32, 0x32, 0x32, 0x34,
                ];
                assert_eq!(wrapped.get_dna().as_slice(), &expected_dna);
            } else {
                panic!("LeaderForgeOutput should have been Forged!")
            }
        });
    }

    #[test]
    fn test_assemble_single_base_with_component_5() {
        ExtBuilder::default().build().execute_with(|| {
            let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

            let mut hash_generators = [
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA3, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA3, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA4, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA4, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA5, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA5, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
            ];

            let progress_arrays = [
                [
                    0x31, 0x20, 0x35, 0x33, 0x30, 0x33, 0x31, 0x32, 0x32, 0x32, 0x34,
                ],
                [
                    0x31, 0x25, 0x32, 0x25, 0x23, 0x22, 0x22, 0x20, 0x23, 0x20, 0x25,
                ],
                [
                    0x22, 0x23, 0x21, 0x22, 0x22, 0x30, 0x22, 0x23, 0x23, 0x22, 0x25,
                ],
                [
                    0x21, 0x21, 0x35, 0x34, 0x24, 0x33, 0x23, 0x22, 0x22, 0x22, 0x22,
                ],
                [
                    0x21, 0x21, 0x35, 0x34, 0x24, 0x33, 0x23, 0x22, 0x35, 0x22, 0x22,
                ],
            ];

            let mut armor_component_set = [
                vec![
                    EquippableItemType::ArmorBase,
                    EquippableItemType::ArmorComponent1,
                ],
                vec![EquippableItemType::ArmorComponent2],
                vec![EquippableItemType::ArmorComponent3],
                vec![EquippableItemType::ArmorComponent1],
                vec![EquippableItemType::ArmorComponent2],
            ]
            .into_iter()
            .zip(progress_arrays)
            .enumerate()
            .map(|(i, (equip_type, progress_array))| {
                let (id, mut avatar) = create_random_armor_component(
                    [0; 32],
                    &ALICE,
                    &PetType::FoxishDude,
                    &SlotType::Head,
                    &RarityTier::Uncommon,
                    equip_type.as_slice(),
                    &(ColorType::Null, ColorType::Null),
                    &Force::Null,
                    i as SoulCount,
                    &mut hash_generators[i],
                );
                avatar.set_progress(progress_array);
                (id, avatar)
            })
            .collect::<Vec<_>>();

            let armor_component_sacrifices = armor_component_set.split_off(1);
            let leader_armor_component = armor_component_set.pop().unwrap();

            let expected_dna = [
                0x41, 0x12, 0x02, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x31, 0x20, 0x35, 0x33, 0x30, 0x33, 0x31,
                0x32, 0x32, 0x32, 0x34,
            ];
            assert_eq!(leader_armor_component.1.get_dna().as_slice(), &expected_dna);

            let (leader_output, _) = AvatarCombinator::<Test>::assemble_avatars(
                leader_armor_component,
                armor_component_sacrifices,
                &mut hash_provider,
            )
            .expect("Should succeed in forging");

            if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
                let wrapped = WrappedAvatar::new(avatar);
                assert_eq!(wrapped.get_souls(), 10);

                let post_assemble = DnaUtils::bits_to_enums::<EquippableItemType>(
                    wrapped.get_spec::<u8>(SpecIdx::Byte1) as u32,
                );
                assert_eq!(post_assemble.len(), 2);
                assert_eq!(post_assemble[0], EquippableItemType::ArmorBase);
                assert_eq!(post_assemble[1], EquippableItemType::ArmorComponent1);

                let leader_progress_array = wrapped.get_progress();
                let expected_leader_progress_array = [
                    0x31, 0x30, 0x35, 0x33, 0x30, 0x33, 0x31, 0x32, 0x32, 0x32, 0x34,
                ];
                assert_eq!(leader_progress_array, expected_leader_progress_array);

                let expected_dna = [
                    0x41, 0x12, 0x03, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x31, 0x30, 0x35, 0x33, 0x30,
                    0x33, 0x31, 0x32, 0x32, 0x32, 0x34,
                ];
                assert_eq!(wrapped.get_dna().as_slice(), &expected_dna);
            } else {
                panic!("LeaderForgeOutput should have been Forged!")
            }
        });
    }

    #[test]
    fn test_assemble_toolbox() {
        ExtBuilder::default().build().execute_with(|| {
            let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

            let mut hash_generators = [
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
                HashProvider::new_with_bytes([
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2, 0xAA, 0xAA, 0xAA, 0xAA,
                    0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                ]),
            ];

            let progress_arrays = [
                [
                    0x31, 0x20, 0x35, 0x33, 0x20, 0x33, 0x31, 0x22, 0x32, 0x22, 0x34,
                ],
                [
                    0x31, 0x25, 0x32, 0x25, 0x21, 0x22, 0x22, 0x21, 0x23, 0x21, 0x25,
                ],
            ];

            let leader = {
                let (id, mut avatar) = create_random_armor_component(
                    [0; 32],
                    &ALICE,
                    &PetType::FoxishDude,
                    &SlotType::Head,
                    &RarityTier::Uncommon,
                    &[EquippableItemType::ArmorBase],
                    &(ColorType::Null, ColorType::Null),
                    &Force::Null,
                    10,
                    &mut hash_generators[0],
                );
                avatar.set_progress(progress_arrays[0]);
                (id, avatar)
            };

            let sac_1 = {
                let (id, mut avatar) = create_random_armor_component(
                    [0; 32],
                    &ALICE,
                    &PetType::FoxishDude,
                    &SlotType::Head,
                    &RarityTier::Uncommon,
                    &[EquippableItemType::ArmorBase],
                    &(ColorType::Null, ColorType::Null),
                    &Force::Null,
                    10,
                    &mut hash_generators[1],
                );
                avatar.set_progress(progress_arrays[1]);
                (id, avatar)
            };

            let sac_2 = {
                let (id, mut avatar) = create_random_armor_component(
                    [0; 32],
                    &ALICE,
                    &PetType::FoxishDude,
                    &SlotType::Head,
                    &RarityTier::Uncommon,
                    &[EquippableItemType::ArmorBase],
                    &(ColorType::Null, ColorType::Null),
                    &Force::Null,
                    10,
                    &mut hash_generators[1],
                );
                avatar.set_progress(progress_arrays[1]);
                (id, avatar)
            };

            let sac_3 = create_random_toolbox([0; 32], &ALICE, 100);
            let sac_4 = create_random_toolbox([0; 32], &ALICE, 100);

            let total_souls = leader.1.get_souls()
                + sac_1.1.get_souls()
                + sac_2.1.get_souls()
                + sac_3.1.get_souls()
                + sac_4.1.get_souls();

            let expected_dna = [
                0x41, 0x12, 0x02, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x31, 0x20, 0x35, 0x33, 0x20, 0x33, 0x31,
                0x22, 0x32, 0x22, 0x34,
            ];
            assert_eq!(leader.1.get_dna().as_slice(), &expected_dna);

            let (leader_output, _) = AvatarCombinator::<Test>::assemble_avatars(
                leader,
                vec![sac_1, sac_2, sac_3, sac_4],
                &mut hash_provider,
            )
            .expect("Should succeed in forging");

            if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
                let wrapped = WrappedAvatar::new(avatar);
                assert_eq!(wrapped.get_souls(), total_souls);

                let leader_rarity = wrapped.get_rarity();
                assert_eq!(leader_rarity, RarityTier::Rare);

                let leader_progress_array = wrapped.get_progress();
                let expected_leader_progress_array = [
                    0x31, 0x30, 0x35, 0x33, 0x30, 0x33, 0x31, 0x32, 0x32, 0x32, 0x34,
                ];
                assert_eq!(leader_progress_array, expected_leader_progress_array);

                let expected_dna = [
                    0x41, 0x12, 0x03, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x31, 0x30, 0x35, 0x33, 0x30,
                    0x33, 0x31, 0x32, 0x32, 0x32, 0x34,
                ];
                assert_eq!(wrapped.get_dna().as_slice(), &expected_dna);
            } else {
                panic!("LeaderForgeOutput should have been Forged!")
            }
        });
    }

    #[test]
    fn test_assemble_failure() {
        ExtBuilder::default().build().execute_with(|| {
            let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

            let hash_base = [
                [
                    0xE7, 0x46, 0x00, 0xE4, 0xE8, 0x78, 0x12, 0xC4, 0xCA, 0x86, 0x53, 0x7F, 0x36,
                    0x1B, 0x64, 0xA0, 0xC3, 0x6B, 0x5C, 0x5F, 0x13, 0x40, 0xBC, 0xC6, 0x97, 0x12,
                    0x25, 0x48, 0xC5, 0xD9, 0x05, 0xC3,
                ],
                [
                    0x3B, 0x06, 0x56, 0x4C, 0x0C, 0x96, 0x6F, 0x41, 0x28, 0x85, 0x40, 0xEC, 0x53,
                    0xAB, 0xF4, 0xCE, 0xCE, 0x6C, 0x60, 0x81, 0xBE, 0xBC, 0xCF, 0x82, 0xBD, 0x70,
                    0x61, 0x14, 0xA2, 0x5E, 0x1A, 0x13,
                ],
                [
                    0x81, 0xA7, 0xCD, 0x5A, 0x36, 0x51, 0xB8, 0xB6, 0xE8, 0x9F, 0x6C, 0xE4, 0xE3,
                    0x52, 0x15, 0xD0, 0xEB, 0xF5, 0x25, 0x97, 0xA7, 0xD2, 0xE4, 0xC0, 0xDC, 0x7C,
                    0xF3, 0x6F, 0xE0, 0xB3, 0x88, 0x76,
                ],
                [
                    0x1A, 0x1C, 0x41, 0x48, 0x0B, 0x96, 0xF9, 0xDC, 0xDA, 0x7A, 0x40, 0x28, 0x99,
                    0x86, 0x58, 0xC3, 0x6A, 0xD4, 0x7C, 0x66, 0x58, 0xD1, 0x9C, 0x8E, 0x81, 0xCF,
                    0xE5, 0x78, 0x70, 0x68, 0x12, 0x0D,
                ],
                [
                    0x84, 0x78, 0x9A, 0x96, 0x77, 0xA2, 0xCE, 0xC3, 0x0E, 0x3C, 0x29, 0x20, 0x33,
                    0x01, 0x67, 0x9C, 0xF8, 0x4D, 0x03, 0x36, 0x80, 0xB5, 0x37, 0x43, 0x6C, 0x71,
                    0xAA, 0xA9, 0x3D, 0x9F, 0x8C, 0xB8,
                ],
            ];

            let mut hash_generators = [
                HashProvider::new_with_bytes([
                    0x40, 0xBC, 0xC6, 0x97, 0x12, 0x25, 0x48, 0xC5, 0xD9, 0x05, 0xC3, 0x40, 0xBC,
                    0xC6, 0x97, 0x12, 0x25, 0x48, 0xC5, 0xD9, 0x05, 0xC3, 0x40, 0xBC, 0xC6, 0x97,
                    0x12, 0x25, 0x48, 0xC5, 0xD9, 0x05,
                ]),
                HashProvider::new_with_bytes([
                    0xBC, 0xCF, 0x82, 0xBD, 0x70, 0x61, 0x14, 0xA2, 0x5E, 0x1A, 0x13, 0xBC, 0xCF,
                    0x82, 0xBD, 0x70, 0x61, 0x14, 0xA2, 0x5E, 0x1A, 0x13, 0xBC, 0xCF, 0x82, 0xBD,
                    0x70, 0x61, 0x14, 0xA2, 0x5E, 0x1A,
                ]),
                HashProvider::new_with_bytes([
                    0xD2, 0xE4, 0xC0, 0xDC, 0x7C, 0xF3, 0x6F, 0xE0, 0xB3, 0x88, 0x76, 0xD2, 0xE4,
                    0xC0, 0xDC, 0x7C, 0xF3, 0x6F, 0xE0, 0xB3, 0x88, 0x76, 0xD2, 0xE4, 0xC0, 0xDC,
                    0x7C, 0xF3, 0x6F, 0xE0, 0xB3, 0x88,
                ]),
                HashProvider::new_with_bytes([
                    0xD1, 0x9C, 0x8E, 0x81, 0xCF, 0xE5, 0x78, 0x70, 0x68, 0x12, 0x0D, 0xD1, 0x9C,
                    0x8E, 0x81, 0xCF, 0xE5, 0x78, 0x70, 0x68, 0x12, 0x0D, 0xD1, 0x9C, 0x8E, 0x81,
                    0xCF, 0xE5, 0x78, 0x70, 0x68, 0x12,
                ]),
                HashProvider::new_with_bytes([
                    0xB5, 0x37, 0x43, 0x6C, 0x71, 0xAA, 0xA9, 0x3D, 0x9F, 0x8C, 0xB8, 0xB5, 0x37,
                    0x43, 0x6C, 0x71, 0xAA, 0xA9, 0x3D, 0x9F, 0x8C, 0xB8, 0xB5, 0x37, 0x43, 0x6C,
                    0x71, 0xAA, 0xA9, 0x3D, 0x9F, 0x8C,
                ]),
            ];

            let slot_types = [
                SlotType::Head,
                SlotType::Head,
                SlotType::LegBack,
                SlotType::Head,
                SlotType::Head,
            ];

            let mut armor_component_set = hash_base
                .into_iter()
                .zip(slot_types)
                .enumerate()
                .map(|(i, (hash, slot_type))| {
                    create_random_armor_component(
                        hash,
                        &ALICE,
                        &PetType::FoxishDude,
                        &slot_type,
                        &RarityTier::Common,
                        &[EquippableItemType::ArmorBase],
                        &(ColorType::Null, ColorType::Null),
                        &Force::Null,
                        i as SoulCount,
                        &mut hash_generators[i],
                    )
                })
                .collect::<Vec<_>>();

            let total_soul_points = armor_component_set
                .iter()
                .map(|(_, avatar)| avatar.get_souls())
                .sum::<SoulCount>();
            assert_eq!(total_soul_points, 10);

            let armor_component_sacrifices = armor_component_set.split_off(1);
            let leader_armor_component = armor_component_set.pop().unwrap();

            let expected_progress_array = [
                0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13,
            ];
            assert_eq!(
                leader_armor_component.1.get_progress(),
                expected_progress_array
            );

            let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::assemble_avatars(
                leader_armor_component,
                armor_component_sacrifices,
                &mut hash_provider,
            )
            .expect("Should succeed in forging");

            assert_eq!(sacrifice_output.len(), 4);
            assert_eq!(
                sacrifice_output
                    .iter()
                    .filter(|output| is_consumed(output))
                    .count(),
                4
            );
            assert_eq!(
                sacrifice_output
                    .iter()
                    .filter(|output| is_forged(output))
                    .count(),
                0
            );

            assert!(is_leader_forged(&leader_output));
        });
    }

    #[test]
    fn test_assemble_1() {
        ExtBuilder::default().build().execute_with(|| {
            let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

            let hash_base = [
                [
                    0x41, 0x61, 0x03, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x43, 0x43, 0x42, 0x40,
                    0x34, 0x31, 0x41, 0x40, 0x43, 0x45,
                ],
                [
                    0x41, 0x61, 0x03, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x32, 0x33, 0x33, 0x41,
                    0x35, 0x30, 0x35, 0x34, 0x45, 0x31,
                ],
                [
                    0x41, 0x61, 0x03, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x35, 0x34, 0x35, 0x31, 0x35,
                    0x33, 0x33, 0x31, 0x31, 0x33, 0x34,
                ],
                [
                    0x41, 0x61, 0x03, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x32, 0x30, 0x31, 0x30, 0x33,
                    0x32, 0x35, 0x31, 0x30, 0x32, 0x35,
                ],
                [
                    0x43, 0x61, 0x03, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x31, 0x31, 0x33, 0x42, 0x31,
                    0x35, 0x32, 0x33, 0x33, 0x34, 0x32,
                ],
            ];

            let unit_fn = |avatar: Avatar| {
                let mut avatar = avatar;
                avatar.souls = 100;
                WrappedAvatar::new(avatar)
            };

            let leader = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[0]), Some(unit_fn));
            let sac_1 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[1]), Some(unit_fn));
            let sac_2 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[2]), Some(unit_fn));
            let sac_3 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[3]), Some(unit_fn));
            let sac_4 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[4]), Some(unit_fn));

            let leader_progress_array = leader.1.get_progress();
            let lowest_count =
                DnaUtils::lowest_progress_indexes(&leader_progress_array, ByteType::High).len();
            assert_eq!(lowest_count, 3);

            let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::assemble_avatars(
                leader,
                vec![sac_1, sac_2, sac_3, sac_4],
                &mut hash_provider,
            )
            .expect("Should succeed in forging");

            assert_eq!(sacrifice_output.len(), 4);
            assert_eq!(
                sacrifice_output
                    .iter()
                    .filter(|output| is_consumed(output))
                    .count(),
                4
            );
            assert_eq!(
                sacrifice_output
                    .iter()
                    .filter(|output| is_forged(output))
                    .count(),
                0
            );

            if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
                let wrapped = WrappedAvatar::new(avatar);
                let out_leader_progress_array = wrapped.get_progress();
                let out_lowest_count =
                    DnaUtils::lowest_progress_indexes(&out_leader_progress_array, ByteType::High)
                        .len();
                assert_eq!(out_lowest_count, 11);

                let expected_dna = [
                    0x41, 0x61, 0x04, 0x01, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x43, 0x43, 0x42, 0x40,
                    0x44, 0x41, 0x41, 0x40, 0x43, 0x45,
                ];
                assert_eq!(wrapped.get_dna().as_slice(), &expected_dna);
            } else {
                panic!("LeaderForgeOutput should be Forged!");
            }
        });
    }

    #[test]
    fn test_assemble_toolbox_prep_1() {
        ExtBuilder::default().build().execute_with(|| {
            let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

            let hash_base = [
                [
                    0x41, 0x24, 0x04, 0x01, 0x00, 0xcf, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x55, 0x51, 0x41, 0x42,
                    0x55, 0x51, 0x41, 0x41, 0x51, 0x53,
                ],
                [
                    0x41, 0x24, 0x04, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x45, 0x44, 0x40, 0x40, 0x43,
                    0x42, 0x40, 0x40, 0x40, 0x41, 0x45,
                ],
                [
                    0x41, 0x24, 0x04, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x44, 0x42, 0x43, 0x40, 0x43,
                    0x44, 0x41, 0x41, 0x45, 0x43, 0x40,
                ],
                [
                    0x64, 0x00, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
                    0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
                ],
                [
                    0x64, 0x00, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
                    0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
                ],
            ];

            let unit_fn = |avatar: Avatar| {
                let mut avatar = avatar;
                avatar.souls = 100;
                WrappedAvatar::new(avatar)
            };

            let leader = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[0]), Some(unit_fn));
            let sac_1 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[1]), Some(unit_fn));
            let sac_2 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[2]), Some(unit_fn));
            let sac_3 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[3]), Some(unit_fn));
            let sac_4 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[4]), Some(unit_fn));

            let total_souls = leader.1.get_souls()
                + sac_1.1.get_souls()
                + sac_2.1.get_souls()
                + sac_3.1.get_souls()
                + sac_4.1.get_souls();

            assert_eq!(
                ForgerV2::<Test>::determine_forge_type(
                    &leader.1,
                    &[&sac_1.1, &sac_2.1, &sac_3.1, &sac_4.1]
                ),
                ForgeType::Assemble
            );

            let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::assemble_avatars(
                leader,
                vec![sac_1, sac_2, sac_3, sac_4],
                &mut hash_provider,
            )
            .expect("Should succeed in forging");

            assert_eq!(sacrifice_output.len(), 4);
            assert_eq!(
                sacrifice_output
                    .iter()
                    .filter(|output| is_consumed(output))
                    .count(),
                4
            );
            assert_eq!(
                sacrifice_output
                    .iter()
                    .filter(|output| is_forged(output))
                    .count(),
                0
            );

            if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
                assert_eq!(avatar.souls, total_souls);

                let leader_rarity =
                    DnaUtils::read_attribute::<RarityTier>(&avatar, AvatarAttr::RarityTier);
                assert_eq!(leader_rarity, RarityTier::Legendary);
            } else {
                panic!("LeaderForgeOutput should have been Forged!")
            }
        });
    }

    #[test]
    fn test_armor_assemble_prep_1() {
        ExtBuilder::default().build().execute_with(|| {
            let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

            let hash_base = [
                [
                    0x41, 0x32, 0x04, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x42, 0x45, 0x40, 0x43, 0x45,
                    0x41, 0x42, 0x45, 0x43, 0x43, 0x41,
                ],
                [
                    0x41, 0x32, 0x02, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x31, 0x25, 0x35, 0x30, 0x32,
                    0x32, 0x35, 0x30, 0x34, 0x35, 0x34,
                ],
                [
                    0x41, 0x32, 0x03, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x41, 0x43, 0x42, 0x42, 0x44,
                    0x45, 0x44, 0x30, 0x45, 0x41, 0x41,
                ],
                [
                    0x41, 0x32, 0x01, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x24, 0x23, 0x23, 0x23,
                    0x22, 0x22, 0x21, 0x22, 0x21, 0x22,
                ],
                [
                    0x41, 0x32, 0x02, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x31, 0x25, 0x30, 0x31, 0x21,
                    0x20, 0x30, 0x34, 0x32, 0x25, 0x21,
                ],
            ];

            let avatar_fn = |souls: SoulCount| {
                let mutate_fn = move |avatar: Avatar| {
                    let mut avatar = avatar;
                    avatar.souls = souls;
                    WrappedAvatar::new(avatar)
                };

                Some(mutate_fn)
            };

            let leader =
                create_random_avatar::<Test, _>(&ALICE, Some(hash_base[0]), avatar_fn(179));
            let sac_1 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[1]), avatar_fn(150));
            let sac_2 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[2]), avatar_fn(236));
            let sac_3 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[3]), avatar_fn(31));
            let sac_4 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[4]), avatar_fn(73));

            let total_souls = leader.1.get_souls()
                + sac_1.1.get_souls()
                + sac_2.1.get_souls()
                + sac_3.1.get_souls()
                + sac_4.1.get_souls();

            let leader_progress = leader.1.get_progress();
            let lowest_count =
                DnaUtils::lowest_progress_indexes(&leader_progress, ByteType::High).len();
            assert_eq!(lowest_count, 11);

            assert_eq!(
                ForgerV2::<Test>::determine_forge_type(
                    &leader.1,
                    &[&sac_1.1, &sac_2.1, &sac_3.1, &sac_4.1]
                ),
                ForgeType::Assemble
            );

            let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::assemble_avatars(
                leader,
                vec![sac_1, sac_2, sac_3, sac_4],
                &mut hash_provider,
            )
            .expect("Should succeed in forging");

            assert_eq!(sacrifice_output.len(), 4);
            assert_eq!(
                sacrifice_output
                    .iter()
                    .filter(|output| is_consumed(output))
                    .count(),
                4
            );
            assert_eq!(
                sacrifice_output
                    .iter()
                    .filter(|output| is_forged(output))
                    .count(),
                0
            );

            if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
                assert_eq!(avatar.souls, total_souls);

                let leader_progress = DnaUtils::read_progress(&avatar);
                let lowest_count =
                    DnaUtils::lowest_progress_indexes(&leader_progress, ByteType::High).len();
                assert_eq!(lowest_count, 11);

                let expected_dna = [
                    0x41, 0x32, 0x04, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x42, 0x45, 0x40, 0x43, 0x45,
                    0x41, 0x42, 0x45, 0x43, 0x43, 0x41,
                ];
                assert_eq!(avatar.dna.as_slice(), &expected_dna);
            } else {
                panic!("LeaderForgeOutput should have been Forged!")
            }
        });
    }
}
