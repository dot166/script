readonly branch=15
readonly aosp_tag_old=android-15.0.0_r2
readonly aosp_tag=android-15.0.0_r2

user_error() {
    echo $1 >&2
    exit 1
}
