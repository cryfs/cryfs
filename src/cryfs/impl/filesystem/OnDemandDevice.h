#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_ONDEMAND_H_
#define MESSMER_CRYFS_FILESYSTEM_ONDEMAND_H_

#include <fspp/fs_interface/Device.h>
#include "CryDevice.h"

namespace cryfs {
class OnDemandDevice final : public fspp::Device {
public:
    OnDemandDevice(bool delayMount,
                   bool onDemand,
                   std::function<cpputils::unique_ref<cryfs::CryDevice>()> device_creator_func);
    virtual ~OnDemandDevice();

public:
    void onFsAction(std::function<void()> callback);

    statvfs statfs() override;
    boost::optional<cpputils::unique_ref<fspp::Node>> Load(const boost::filesystem::path &path) override;
    boost::optional<cpputils::unique_ref<fspp::File>> LoadFile(const boost::filesystem::path &path) override;
    boost::optional<cpputils::unique_ref<fspp::Dir>> LoadDir(const boost::filesystem::path &path) override;
    boost::optional<cpputils::unique_ref<fspp::Symlink>> LoadSymlink(const boost::filesystem::path &path) override;

    void deref() override;
private:
    bool _delayMount;
    bool _onDemand;
    boost::optional<cpputils::unique_ref<cryfs::CryDevice>> _device;
    std::function<cpputils::unique_ref<cryfs::CryDevice>()> _device_creator_func;
    std::vector<std::function<void()>> _onFsAction;

    void CreateDevice();

    DISALLOW_COPY_AND_ASSIGN(OnDemandDevice);
};

} // namespace cryfs
#endif //MESSMER_CRYFS_FILESYSTEM_ONDEMAND_H_
