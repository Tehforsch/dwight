id=RPI-RP2
echo $id
mountPath=/run/media/toni/$id
devPart=$(sudo blkid | grep $id | cut -d ":" -f 1)
if [[ $devPart == "" ]]; then
    echo "partition not found"
    exit 1
fi
echo "mounting $devPart"
echo mkdir -p $mountPath
sudo mkdir -p $mountPath
sudo umount $mountPath
sudo mount $devPart $mountPath
sudo chmod ugo+rwx $mountPath
sudo chmod -R ugo+rwx $mountPath/
cargo run --release
